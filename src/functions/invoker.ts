import type { FilterValues, NumericalString, Media, Tag } from '../type'
import { invoke } from '@tauri-apps/api/core'
import { normalizeInvokeError } from './errorMessage'
import { logFrontendError } from './errorHandling'

async function invokeCommand<T>(command: string, payload?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, payload)
  } catch (error: unknown) {
    logFrontendError(`invoke.${command}`, error)
    throw normalizeInvokeError(error, `Command failed: ${command}`)
  }
}

export async function sync_files(dir: string): Promise<number> {
  return await invokeCommand<number>('sync_files', { root: dir })
}

export async function get_people(): Promise<NumericalString[]> {
  return await invokeCommand<NumericalString[]>('get_people')
}

export async function get_genres(): Promise<NumericalString[]> {
  return await invokeCommand<NumericalString[]>('get_genres')
}

export async function get_countries(): Promise<NumericalString[]> {
  return await invokeCommand<NumericalString[]>('get_countries')
}

export async function filter_medias(filters: FilterValues, page: number): Promise<Media[]> {
  return await invokeCommand<Media[]>('filter_medias', { filters, page })
}

export async function get_media_by_id(mediaId: number): Promise<Media> {
  return await invokeCommand<Media>('get_media_by_id', { mediaId })
}

export async function update_media_imdb(mediaId: number, imdbId: string): Promise<number> {
  return await invokeCommand<number>('update_media_imdb', { mediaId, imdbId })
}

export async function create_media_from_imdb(imdbId: string): Promise<number> {
  return await invokeCommand<number>('create_media_from_imdb', { imdbId })
}

export async function update_media_watched(mediaId: number, watched: boolean): Promise<void> {
  return await invokeCommand<void>('update_media_watched', { mediaId, watched })
}

export async function update_season_watched(seasonId: number, watched: boolean): Promise<void> {
  return await invokeCommand<void>('update_season_watched', { seasonId, watched })
}

export async function update_episode_watched(episodeId: number, watched: boolean): Promise<void> {
  return await invokeCommand<void>('update_episode_watched', { episodeId, watched })
}

export async function update_media_my_ranking(mediaId: number, myRanking: number): Promise<void> {
  return await invokeCommand<void>('update_media_my_ranking', { mediaId, myRanking })
}

export async function update_media_watch_list(mediaId: number, watchList: boolean) {
  return await invokeCommand('update_watch_list', { mediaId, watchList })
}

export async function get_tags(): Promise<Tag[]> {
  return await invokeCommand<Tag[]>('get_tags')
}

export async function remove_tag(tagId: number): Promise<void> {
  return await invokeCommand<void>('remove_tag', { tagId })
}

export async function update_tag(tag: Tag): Promise<void> {
  return await invokeCommand<void>('update_tag', { tag })
}

export async function get_medias_by_tag(tagId: number): Promise<Media> {
  return await invokeCommand<Media>('get_medias_by_tag', { tagId })
}

export async function insert_tag(tag: Tag): Promise<void> {
  return await invokeCommand<void>('insert_tag', { tag })
}

export async function insert_media_tag(mediaId: number, tagId: number): Promise<void> {
  return await invokeCommand<void>('insert_media_tag', { mediaId, tagId })
}

export async function remove_media_tag(mediaId: number, tagId: number): Promise<void> {
  return await invokeCommand<void>('remove_media_tag', { mediaId, tagId })
}

export async function delete_media(mediaId: number): Promise<void> {
  return await invokeCommand<void>('delete_media', { mediaId })
}

export async function export_data(filePath: string): Promise<void> {
  return await invokeCommand<void>('export_data', { filePath })
}

export async function import_data(data: string): Promise<void> {
  return await invokeCommand<void>('import_data', { data })
}
