export interface MovieSearchResult {
  ok: boolean
  description: SearchedMovie[]
  error_code: number
}

export interface SearchedMovie {
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
