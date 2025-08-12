import type { FilterValues, NumericalString, VideoMetaData } from '../type'
import { invoke } from '@tauri-apps/api/core'

export async function create_table() {
  await invoke('create_table_app')
}

export async function sync_app(dir: string): Promise<number> {
  return await invoke<number>('sync_app_files', { root: dir, apiKey: '4c602a26' })
}

export async function get_actors() {
  return await invoke<NumericalString[]>('get_actors_app')
}

export async function get_genres() {
  return await invoke<NumericalString[]>('get_genres_app')
}

export async function get_countries() {
  return await invoke<NumericalString[]>('get_countries_app')
}

export async function search_videos(filters: FilterValues): Promise<VideoMetaData[]> {
  return await invoke<VideoMetaData[]>('search_videos_app', { filters })
}

export async function get_video_by_id(videoId: number): Promise<VideoMetaData> {
  return await invoke('get_video_by_id_app', { videoId })
}

export async function update_video_imdb(videoId: number, imdbId: string): Promise<void> {
  return await invoke('update_video_imdb_app', { videoId, imdbId, apiKey: '4c602a26' })
}

export async function update_video_watched(videoId: number, watched: boolean) {
  return await invoke('update_video_watched_app', { videoId, watched })
}

export async function update_video_my_ranking(videoId: number, myRanking: number) {
  return await invoke('update_video_my_ranking_app', { videoId, myRanking })
}
