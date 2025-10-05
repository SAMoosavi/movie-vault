import { Store } from '@tauri-apps/plugin-store'
import { check, Update } from '@tauri-apps/plugin-updater'
import { toast } from 'vue3-toastify'

// Settings store
let settingsStore: Store | null = null

// Initialize store
export async function initUpdateStore(): Promise<Store> {
  if (!settingsStore) {
    settingsStore = await Store.load('settings.json', {
      defaults: {
        autoUpdate: false,
        betaVersions: false,
      },
      autoSave: true,
    })
  }
  return settingsStore
}

// Load update settings
export async function loadUpdateSettings(): Promise<{ autoUpdate: boolean; betaVersions: boolean }> {
  const store = await initUpdateStore()
  const autoUpdate = (await store.get<boolean>('autoUpdate')) ?? false
  const betaVersions = (await store.get<boolean>('betaVersions')) ?? false
  return { autoUpdate, betaVersions }
}

// Save update settings
export async function saveUpdateSettings(settings: { autoUpdate?: boolean; betaVersions?: boolean }) {
  const store = await initUpdateStore()
  if (settings.autoUpdate !== undefined) {
    await store.set('autoUpdate', settings.autoUpdate)
  }
  if (settings.betaVersions !== undefined) {
    await store.set('betaVersions', settings.betaVersions)
  }
}

// Check for updates
export async function checkForUpdates(): Promise<Update | null> {
  try {
    const update = await check()
    return update
  } catch (error) {
    console.error('Failed to check for updates:', error)
    return null
  }
}

// Install update
export async function installAppUpdate(update: Update): Promise<void> {
  try {
    await update.downloadAndInstall()
  } catch (error) {
    console.error('Failed to install update:', error)
    throw error
  }
}

// Handle update found
export async function handleUpdateFound(update: Update) {
  const { autoUpdate } = await loadUpdateSettings()

  if (autoUpdate) {
    // Auto download and prompt to restart
    toast.info(`Update ${update.version} available. Downloading...`, {
      autoClose: 3000,
    })
    try {
      await installAppUpdate(update)
      toast.success(`Update ${update.version} downloaded. Please restart the app.`, {
        autoClose: false,
        onClick: () => {
          // Restart app
          window.location.reload()
        },
      })
    } catch (error) {
      console.error('Failed to download update:', error)
      toast.error('Failed to download update.')
    }
  } else {
    // Show toast to accept
    toast.info(`Update ${update.version} available. Click to install.`, {
      autoClose: false,
      onClick: async () => {
        try {
          await installAppUpdate(update)
          toast.success(`Update ${update.version} downloaded. Please restart the app.`, {
            autoClose: false,
            onClick: () => {
              window.location.reload()
            },
          })
        } catch (error) {
          console.error('Failed to download update:', error)
          toast.error('Failed to download update.')
        }
      },
    })
  }
}
