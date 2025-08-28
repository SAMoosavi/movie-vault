import type { FilterValues, NumericalString, Media, Tag } from '../type'
import { invoke } from '@tauri-apps/api/core'

export async function sync_files(dir: string): Promise<number> {
  return await invoke('sync_files', { root: dir, apiKey: '229a288b' })
}

export async function get_actors(): Promise<NumericalString[]> {
  return await invoke('get_actors')
}

export async function get_genres(): Promise<NumericalString[]> {
  return await invoke('get_genres')
}

export async function get_countries(): Promise<NumericalString[]> {
  return await invoke('get_countries')
}

export async function filter_medias(filters: FilterValues): Promise<Media[]> {
  return await invoke('filter_medias', { filters })
}

export async function get_media_by_id(mediaId: number): Promise<Media> {
  return await invoke('get_media_by_id', { mediaId })
}

export async function update_media_imdb(mediaId: number, imdbId: string): Promise<number> {
  return await invoke('update_media_imdb', { mediaId, imdbId, apiKey: '229a288b' })
}

export async function update_media_watched(mediaId: number, watched: boolean): Promise<void> {
  return await invoke('update_media_watched', { mediaId, watched })
}

export async function update_season_watched(seasonId: number, watched: boolean): Promise<void> {
  return await invoke('update_season_watched', { seasonId, watched })
}

export async function update_episode_watched(episodeId: number, watched: boolean): Promise<void> {
  return await invoke('update_episode_watched', { episodeId, watched })
}

export async function update_media_my_ranking(mediaId: number, myRanking: number): Promise<void> {
  return await invoke('update_media_my_ranking', { mediaId, myRanking })
}

export async function update_media_watch_list(mediaId: number, watchList: boolean) {
  return await invoke('update_watch_list', { mediaId, watchList })
}

export async function get_tags(): Promise<Tag[]> {
  return await invoke('get_tags')
}

export async function remove_tag(tagId: number): Promise<void> {
  return await invoke('remove_tag', { tagId })
}

export async function update_tag(tag: Tag): Promise<void> {
  return await invoke('update_tag', { tag })
}

export async function get_medias_by_tag(tagId: number): Promise<Media> {
  return await invoke('get_medias_by_tag', { tagId })
}

export async function insert_tag(tag: Tag): Promise<void> {
  return await invoke('insert_tag', { tag })
}

export async function insert_media_tag(mediaId: number, tagId: number): Promise<void> {
  return await invoke('insert_media_tag', { mediaId, tagId })
}

export async function remove_media_tag(mediaId: number, tagId: number): Promise<void> {
  return await invoke('remove_media_tag', { mediaId, tagId })
}
