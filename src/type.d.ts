export interface SeriesMeta {
  season: number
  episode: number
}

export interface VideoFileData {
  title: string
  path: string
  quality?: string
  language_format: string
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
  subtitle_path?: string
  year?: number
  files: VideoFileData[]
  series?: SeriesMeta
  imdb?: Imdb
  watched: boolean
  my_ranking: number
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
}
