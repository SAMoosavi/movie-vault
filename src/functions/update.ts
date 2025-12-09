import { Store } from '@tauri-apps/plugin-store'
import { check, Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { getVersion } from '@tauri-apps/api/app'
import { info, error } from '@tauri-apps/plugin-log'
import { toast } from 'vue3-toastify'

// Types
interface UpdateSettings {
  autoUpdate: boolean
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
    info(`Auto-update ${enabled ? 'enabled' : 'disabled'}`)
  } catch (err) {
    error(`Failed to save auto-update setting: ${err}`)
    throw err
  }
}

// Update check
export async function checkForUpdates(): Promise<Update | null> {
  try {
    info('Checking for updates...')
    const update = await check()

    if (!update?.available) {
      info('No updates available')
      return null
    }

    const currentVersion = await getVersion()
    info(`Current: ${currentVersion}, Available: ${update.version}`)

    return update
  } catch (err) {
    error(`Update check failed: ${err}`)
    return null
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
export async function handleUpdateCheck(): Promise<void> {
  const update = await checkForUpdates()
  if (!update) return

  const { autoUpdate } = await getUpdateSettings()
  const version = update.version

  if (autoUpdate) {
    toast.info(`Installing update ${version}...`, { autoClose: false })

    try {
      await installUpdate(update)
    } catch (err) {
      error(`Auto-update failed: ${err}`)
      toast.error('Update failed. Please try again later.')
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
          error(`Manual update failed: ${err}`)
          toast.error('Update failed. Please try again.')
        }
      },
    })
  }
}
