export interface Tag {
  id: number
  name: string
  color: string
}

export interface File {
  id: number
  file_name: string
  path: string
  quality?: string
  language_format: string
}

export interface Episode {
  id: number
  number: number
  watched: boolean
  files: File[]
}

export interface Season {
  id: number
  number: number
  watched: boolean
  episodes: Episode[]
}

export interface Imdb {
  title: string
  year: string
  rated: string
  released: string
  runtime: string
  genre: string[]
  directors: string[]
  writers: string[]
  actors: string[]
  plot: string
  languages: string[]
  country: string[]
  awards: string
  poster: string
  imdb_rating: string
  imdb_votes: string
  imdb_id: string
  box_office?: string
  total_seasons?: string
  type: string
}

export interface Media {
  id: number
  name: string
  year?: number
  files: File[]
  seasons: Season[]
  imdb?: Imdb
  watched: boolean
  my_ranking: number
  watch_list: boolean
  tags: Tag[]
}

type ContentType = 'all' | 'movie' | 'series'
type SortByType = 'name' | 'year' | 'imdb'
type SortDirectionType = 'asc' | 'desc'
type NullableBool = boolean | null
type NullableNumber = number | null
type NumericalString = [number, string]

export interface FilterValues {
  type: ContentType
  minRating: NullableNumber
  country: NumericalString[]
  genre: NumericalString[]
  name: string
  existImdb: NullableBool
  existMultiFile: NullableBool
  actor: NumericalString[]
  watched: NullableBool
  sortBy: SortByType
  sortDirection: SortDirectionType
  watchList: NullableBool
}
