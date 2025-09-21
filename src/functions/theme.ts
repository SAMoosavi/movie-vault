import { Store } from '@tauri-apps/plugin-store'

// List of available theme names
export const themes = [
  'dracula',
  'winter',
  'lemonade',
  'light',
  'dark',
  'cupcake',
  'bumblebee',
  'emerald',
  'corporate',
  'synthwave',
  'retro',
  'cyberpunk',
  'valentine',
  'aqua',
  'lofi',
  'pastel',
  'forest',
  'black',
  'luxury',
  'night',
]

// Returns the default theme based on user's system preference
export function getDefaultTheme(): string {
  // If user prefers dark mode, use the first theme ('dracula')
  // Otherwise, use the second theme ('winter')
  const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
  return prefersDark ? themes[0]! : themes[1]!
}

// Sets the theme for the document and saves it in the store
export async function setTheme(theme: string, store: Store) {
  document.documentElement.setAttribute('data-theme', theme)
  await store.set('theme', theme)
}

// Initializes and returns the settings store
export async function initStore(): Promise<Store> {
  // Loads 'settings.json' with autoSave enabled
  return await Store.load('settings.json', { defaults: { theme: 'light' }, autoSave: true })
}

// Loads the saved theme from the store, if any
export async function loadTheme(store: Store): Promise<string | undefined> {
  return await store.get<string>('theme')
}
