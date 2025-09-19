export interface MediaSearchResult {
  ok: boolean
  description: SearchedMedia[]
  error_code: number
}

export interface SearchedMedia {
  '#TITLE': string
  '#YEAR': string | number
  '#IMDB_ID': string
  '#RANK': number
  '#ACTORS': string
  '#AKA': string
  '#IMDB_URL': string
  '#IMDB_IV': string
  '#IMG_POSTER'?: string
  photo_width?: number
  photo_height?: number
}
