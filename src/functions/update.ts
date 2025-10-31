import { Store } from '@tauri-apps/plugin-store'
import { check, Update } from '@tauri-apps/plugin-updater'
import { toast } from 'vue3-toastify'
import * as p from '../../package.json'

// Settings store
let settingsStore: Store | null = null

const __APP_VERSION__ = p.version

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
  console.log('[Updater] Checking for updates...')
  try {
    const update = await check()
    if (!update) {
      console.log('[Updater] No updates found.')
      return null
    }

    console.log(`[Updater] Latest version found: ${update.version}`)
    if (update.version === undefined || update.version.trim() === '') {
      console.error('[Updater] Invalid version received from updater API.')
      return null
    }

    // Basic semantic version comparison (fallback)
    const currentVersion = __APP_VERSION__ || '0.0.0'
    console.log(`[Updater] Current app version: ${currentVersion}`)
    if (update.version === currentVersion) {
      console.log('[Updater] App is up to date.')
      return null
    }

    console.log(`[Updater] Update available: ${update.version}`)
    return update
  } catch (error) {
    console.error('[Updater] Failed to check for updates:', error)
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
  console.log(`[Updater] Handling update for version: ${update?.version}`)

  if (!update || !update.version) {
    console.error('[Updater] Invalid update object received.')
    toast.error('Error: Invalid update data received.')
    return
  }

  if (autoUpdate) {
    toast.info(`Auto-updating to ${update.version}...`, { autoClose: 3000 })
    try {
      await installAppUpdate(update)
      console.log(`[Updater] Update ${update.version} downloaded successfully.`)
      toast.success(`Update ${update.version} installed. Restarting...`, {
        autoClose: false,
        onClick: () => window.location.reload(),
      })
    } catch (error) {
      console.error('[Updater] Auto-update failed:', error)
      toast.error('Auto-update failed. Please try manual update.')
    }
  } else {
    console.log(`[Updater] Manual update available: ${update.version}`)
    toast.info(`Update ${update.version} available. Click to install.`, {
      autoClose: false,
      onClick: async () => {
        console.log(`[Updater] Starting manual install for ${update.version}`)
        try {
          await installAppUpdate(update)
          toast.success(`Update ${update.version} installed. Restarting...`, {
            autoClose: false,
            onClick: () => window.location.reload(),
          })
        } catch (error) {
          console.error('[Updater] Manual update failed:', error)
          toast.error('Manual update failed.')
        }
      },
    })
  }
}
