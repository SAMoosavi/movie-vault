import type { FilterValues, VideoMetaData } from '../type'
import { invoke } from '@tauri-apps/api/core'

export async function create_table() {
  await invoke('create_table_app')
}

export async function sync_app(dir: string): Promise<number> {
  return await invoke<number>('sync_app_files', { root: dir, apiKey: '4c602a26' })
}

export async function get_all_video_metadata() {
  return await invoke<VideoMetaData[]>('get_all_video_metadata_app')
}

export async function get_genres() {
  return await invoke<[number, string][]>('get_genres_app')
}

export async function get_countries() {
  return await invoke<[number, string][]>('get_countries_app')
}

export async function search_videos(filters: FilterValues): Promise<VideoMetaData[]> {
  return await invoke<VideoMetaData[]>('search_videos_app', { filters })
}

export async function get_video_by_id(videoId: number): Promise<VideoMetaData> {
  return await invoke('get_video_by_id_app', { videoId })
}
