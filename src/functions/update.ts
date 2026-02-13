import { Store } from '@tauri-apps/plugin-store'
import { check, Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { getVersion } from '@tauri-apps/api/app'
import { info, error } from '@tauri-apps/plugin-log'
import { toast } from 'vue3-toastify'
import { getErrorMessage } from '@/functions/errorMessage'

// Types
interface UpdateSettings {
  autoUpdate: boolean
}

interface HandleUpdateCheckOptions {
  notifyIfUpToDate?: boolean
}

// Store singleton with lazy initialization
let storeInstance: Store | null = null
let storePromise: Promise<Store> | null = null

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
    error(`Failed to load update settings: ${err}`)
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
    const message = getErrorMessage(err)
    error(`Failed to save auto-update setting: ${message}`)
    throw new Error(message)
  }
}

// Update check
export async function checkForUpdates(): Promise<Update | null> {
  try {
    info('Checking for updates...')
    const update = await check()

    if (!update) {
      info('No updates available')
      return null
    }

    const currentVersion = await getVersion()
    info(`Current: ${currentVersion}, Available: ${update.version}`)

    return update
  } catch (err) {
    const message = getErrorMessage(err)
    error(`Update check failed: ${message}`)
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
      const message = getErrorMessage(err)
      error(`Auto-update failed: ${message}`)
      toast.error('Update failed: ' + message)
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
          const message = getErrorMessage(err)
          error(`Manual update failed: ${message}`)
          toast.error('Update failed: ' + message)
        }
      },
    })
  }
}
