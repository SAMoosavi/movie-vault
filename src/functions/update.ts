import { Store } from '@tauri-apps/plugin-store'
import { check, Update } from '@tauri-apps/plugin-updater'
import { toast } from 'vue3-toastify'
import * as p from '../../package.json'
import { info, error } from '@tauri-apps/plugin-log'

// Settings store
let settingsStore: Store | null = null

const __APP_VERSION__ = p.version

// Initialize store
export async function initUpdateStore(): Promise<Store> {
  if (settingsStore) {
    info('Update store already initialized.')
    return settingsStore
  }

  try {
    info('Loading update settings store...')
    settingsStore = await Store.load('settings.json', {
      defaults: {
        autoUpdate: false,
        betaVersions: false,
      },
      autoSave: true,
    })
    info('Update settings store loaded.')
    return settingsStore
  } catch (err) {
    error(`Failed to load update settings store: ${err}`)
    throw err
  }
}

// Load update settings
export async function loadUpdateSettings(): Promise<{ autoUpdate: boolean; betaVersions: boolean }> {
  try {
    const store = await initUpdateStore()
    const autoUpdate = (await store.get<boolean>('autoUpdate')) ?? false
    const betaVersions = (await store.get<boolean>('betaVersions')) ?? false
    info(`Loaded update settings: autoUpdate=${autoUpdate}, betaVersions=${betaVersions}`)
    return { autoUpdate, betaVersions }
  } catch (err) {
    error(`Failed to load update settings, using defaults:${err}`)
    return { autoUpdate: false, betaVersions: false }
  }
}

// Save update settings
export async function saveUpdateSettings(settings: { autoUpdate?: boolean; betaVersions?: boolean }) {
  try {
    const store = await initUpdateStore()
    if (settings.autoUpdate !== undefined) {
      await store.set('autoUpdate', settings.autoUpdate)
      info(`Updated autoUpdate setting to ${settings.autoUpdate}`)
    }
    if (settings.betaVersions !== undefined) {
      await store.set('betaVersions', settings.betaVersions)
      info(`Updated betaVersions setting to ${settings.betaVersions}`)
    }
    info('Update settings saved successfully.')
  } catch (err) {
    error(`Failed to save update settings: ${err}`)
    throw err
  }
}

// Check for updates
export async function checkForUpdates(): Promise<Update | null> {
  info('Checking for updates...')
  try {
    const update = await check()
    if (!update) {
      info('No updates found.')
      return null
    }

    info(`Latest version found: ${update.version}`)
    if (update.version === undefined || update.version.trim() === '') {
      error('Invalid version received from updater API.')
      return null
    }

    // Basic semantic version comparison (fallback)
    const currentVersion = __APP_VERSION__ || '0.0.0'
    info(`Current app version: ${currentVersion}`)
    if (update.version === currentVersion) {
      info('App is up to date.')
      return null
    }

    info(`Update available: ${update.version}`)
    return update
  } catch (err) {
    error(`Failed to check for updates: ${err}`)
    return null
  }
}

// Install update
export async function installAppUpdate(update: Update): Promise<void> {
  try {
    info('Starting app update installation...')
    await update.downloadAndInstall()
    info('App update installed successfully.')
  } catch (err) {
    error(`Failed to install update: ${err}`)
    throw err
  }
}

// Handle update found
export async function handleUpdateFound(update: Update) {
  const { autoUpdate } = await loadUpdateSettings()
  info(`Handling update for version: ${update?.version}`)

  if (!update || !update.version) {
    error('Invalid update object received - update object is null or missing version.')
    toast.error('Invalid update data received.')
    return
  }

  if (autoUpdate) {
    info(`Auto-update enabled. Starting update to version ${update.version}`)
    toast.info(`Auto-updating to ${update.version}...`, { autoClose: 3000 })
    try {
      await installAppUpdate(update)
      info(`Update ${update.version} downloaded and installed successfully.`)
      toast.success(`Update ${update.version} installed. Restarting...`, {
        autoClose: false,
        onClick: () => window.location.reload(),
      })
    } catch (err) {
      error(`Auto-update failed for version ${update.version}: ${err}`)
      toast.error('Auto-update failed. Please try manual update.')
    }
  } else {
    info(`Auto-update disabled. Manual update available for version ${update.version}`)
    toast.info(`Update ${update.version} available. Click to install.`, {
      autoClose: false,
      onClick: async () => {
        info(`User initiated manual update for version ${update.version}`)
        try {
          await installAppUpdate(update)
          info(`Manual update to version ${update.version} completed successfully.`)
          toast.success(`Update ${update.version} installed. Restarting...`, {
            autoClose: false,
            onClick: () => window.location.reload(),
          })
        } catch (err) {
          error(`Manual update failed for version ${update.version}: ${err}`)
          toast.error('Manual update failed.')
        }
      },
    })
  }
}
