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

export async function filter_medias(filters: FilterValues): Promise<VideoMetaData[]> {
  return await invoke<VideoMetaData[]>('filter_medias_app', { filters })
}

export async function get_media_by_id(mediaId: number): Promise<VideoMetaData> {
  return await invoke('get_media_by_id_app', { mediaId })
}

export async function update_media_imdb(mediaId: number, imdbId: string): Promise<void> {
  return await invoke('update_media_imdb_app', { mediaId, imdbId, apiKey: '4c602a26' })
}

export async function update_media_watched(mediaId: number, watched: boolean) {
  return await invoke('update_media_watched_app', { mediaId, watched })
}

export async function update_season_watched(seasonId: number, watched: boolean) {
  return await invoke('update_season_watched_app', { seasonId, watched })
}

export async function update_episode_watched(episodeId: number, watched: boolean) {
  return await invoke('update_episode_watched_app', { episodeId, watched })
}

export async function update_media_my_ranking(mediaId: number, myRanking: number) {
  return await invoke('update_media_my_ranking_app', { mediaId, myRanking })
}
