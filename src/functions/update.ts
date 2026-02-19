import { Store } from '@tauri-apps/plugin-store'
import { check, Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { getVersion } from '@tauri-apps/api/app'
import { info } from '@tauri-apps/plugin-log'
import { toast } from 'vue3-toastify'
import { handleFrontendError, logFrontendError } from '@/functions/errorHandling'

// Types
interface UpdateSettings {
  autoUpdate: boolean
}

interface HandleUpdateCheckOptions {
  notifyIfUpToDate?: boolean
}

const TARGETS_NOT_FOUND_PATTERN = /fallback platforms `(\[[^\]]*])`/
const LINUX_INSTALLERS = ['appimage', 'deb', 'rpm']
const WINDOWS_INSTALLERS = ['nsis', 'msi']
const DARWIN_INSTALLERS = ['app']

// Store singleton with lazy initialization
let storeInstance: Store | null = null
let storePromise: Promise<Store> | null = null

function installersForOs(os: string): string[] {
  if (os === 'linux') return LINUX_INSTALLERS
  if (os === 'windows') return WINDOWS_INSTALLERS
  if (os === 'darwin') return DARWIN_INSTALLERS
  return []
}

function addTargetWithArchAliases(target: string, targets: Set<string>) {
  targets.add(target)
  if (target.includes('x86_64')) {
    targets.add(target.replace('x86_64', 'x64'))
  } else if (target.includes('x64')) {
    targets.add(target.replace('x64', 'x86_64'))
  }
}

function extractMissingTargets(error: unknown): string[] {
  const message = error instanceof Error ? error.message : String(error)
  const match = message.match(TARGETS_NOT_FOUND_PATTERN)
  if (!match) return []

  try {
    const serializedTargets = match[1]
    if (!serializedTargets) return []
    const parsed = JSON.parse(serializedTargets)
    if (!Array.isArray(parsed)) return []
    return parsed.filter((item): item is string => typeof item === 'string' && item.length > 0)
  } catch {
    return []
  }
}

function buildRetryTargets(baseTargets: string[]): string[] {
  const candidates = new Set<string>()

  for (const target of baseTargets) {
    addTargetWithArchAliases(target, candidates)

    const parts = target.split('-')
    if (parts.length === 2) {
      const [os, arch] = parts
      if (!os || !arch) continue
      const installers = installersForOs(os)
      for (const installer of installers) {
        addTargetWithArchAliases(`${os}-${arch}-${installer}`, candidates)
        if (os === 'linux') {
          addTargetWithArchAliases(`${os}-${installer}-${arch}`, candidates)
        }
      }
      continue
    }

    if (parts.length !== 3) continue
    const [os, second, third] = parts
    if (!os || !second || !third) continue
    const installers = installersForOs(os)

    // Supports current format: os-arch-installer
    if (installers.includes(third)) {
      addTargetWithArchAliases(`${os}-${second}`, candidates)
      if (os === 'linux') {
        addTargetWithArchAliases(`${os}-${third}-${second}`, candidates)
      }
    }

    // Supports legacy format: os-installer-arch
    if (installers.includes(second)) {
      addTargetWithArchAliases(`${os}-${third}`, candidates)
      addTargetWithArchAliases(`${os}-${third}-${second}`, candidates)
    }
  }

  return Array.from(candidates).filter((target) => !baseTargets.includes(target))
}

async function checkWithTargetFallbacks(): Promise<Update | null> {
  try {
    return await check()
  } catch (error) {
    const missingTargets = extractMissingTargets(error)
    if (missingTargets.length === 0) {
      throw error
    }

    const retryTargets = buildRetryTargets(missingTargets)
    if (retryTargets.length === 0) {
      throw error
    }

    info(`Updater targets missing (${missingTargets.join(', ')}). Retrying with: ${retryTargets.join(', ')}`)

    let lastError: unknown = error
    for (const target of retryTargets) {
      try {
        info(`Retrying update check with explicit target: ${target}`)
        return await check({ target })
      } catch (retryError) {
        lastError = retryError
      }
    }

    throw lastError
  }
}

async function getStore(): Promise<Store> {
  if (storeInstance) return storeInstance

  if (!storePromise) {
    storePromise = Store.load('settings.json').then((store) => {
      storeInstance = store
      info('Update settings store initialized')
      return store
    })
  }

  return storePromise
}

// Settings
export async function getUpdateSettings(): Promise<UpdateSettings> {
  try {
    const store = await getStore()
    const autoUpdate = (await store.get<boolean>('autoUpdate')) ?? false
    return { autoUpdate }
  } catch (err) {
    logFrontendError('update.settings.load', err)
    return { autoUpdate: false }
  }
}

export async function setAutoUpdate(enabled: boolean): Promise<void> {
  try {
    const store = await getStore()
    await store.set('autoUpdate', enabled)
    await store.save()
    info(`Auto-update ${enabled ? 'enabled' : 'disabled'}`)
  } catch (err) {
    const message = logFrontendError('update.settings.save', err)
    throw new Error(message)
  }
}

// Update check
export async function checkForUpdates(): Promise<Update | null> {
  try {
    info('Checking for updates...')
    const update = await checkWithTargetFallbacks()

    if (!update) {
      info('No updates available')
      return null
    }

    const currentVersion = await getVersion()
    info(`Current: ${currentVersion}, Available: ${update.version}`)

    return update
  } catch (err) {
    const message = logFrontendError('update.check', err)
    throw new Error(message)
  }
}

// Update installation
async function installUpdate(update: Update): Promise<void> {
  info(`Installing update ${update.version}...`)

  await update.downloadAndInstall((progress) => {
    if (progress.event === 'Started' && progress.data.contentLength) {
      info(`Download started: ${progress.data.contentLength} bytes`)
    } else if (progress.event === 'Progress') {
      info(`Downloaded ${progress.data.chunkLength} bytes`)
    } else if (progress.event === 'Finished') {
      info('Download complete')
    }
  })

  info('Update installed, relaunching...')
  await relaunch()
}

// Main update handler
export async function handleUpdateCheck(options: HandleUpdateCheckOptions = {}): Promise<void> {
  const update = await checkForUpdates()
  if (!update) {
    if (options.notifyIfUpToDate) {
      toast.info('You are using the latest version.')
    }
    return
  }

  const { autoUpdate } = await getUpdateSettings()
  const version = update.version

  if (autoUpdate) {
    toast.info(`Installing update ${version}...`, { autoClose: false })

    try {
      await installUpdate(update)
    } catch (err) {
      handleFrontendError('update.install.auto', err, 'Update failed')
    }
  } else {
    toast.info(`Update ${version} available`, {
      autoClose: false,
      closeOnClick: false,
      onClick: async () => {
        toast.info('Installing update...', { autoClose: false })

        try {
          await installUpdate(update)
        } catch (err) {
          handleFrontendError('update.install.manual', err, 'Update failed')
        }
      },
    })
  }
}
