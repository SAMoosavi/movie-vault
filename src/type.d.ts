export interface Tag {
  id: number
  name: string
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

export interface Actor {
  id: number
  name: string
  url: string
}

export interface Imdb {
  title: string
  year: string
  released: string
  genres: string[]
  actors: Actor[]
  plot: string
  countries: string[]
  poster: string
  imdb_rating: string
  imdb_votes: string
  imdb_id: string
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

type ContentType = 'all' | 'Movie' | 'TVSeries'
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
  tags: NumericalString[]
}
