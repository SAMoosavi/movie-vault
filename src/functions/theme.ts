import { Store } from '@tauri-apps/plugin-store'

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

export function getDefaultTheme(): string {
  const num = window.matchMedia('(prefers-color-scheme: dark)').matches ? 0 : 1
  return themes[num]
}

export async function setTheme(theme: string, store: Store) {
  document.documentElement.setAttribute('data-theme', theme)
  await store.set('theme', theme)
}

export async function initStore(): Promise<Store> {
  return await Store.load('settings.json', { autoSave: true })
}

export async function loadTheme(store: Store): Promise<string | undefined> {
  return await store.get<string>('theme')
}
