export interface SeriesMeta {
  season: number
  episode: number
}

export interface VideoFileData {
  title: string
  path: string // PathBuf as string
  quality?: string
  has_hard_sub: boolean
  has_soft_sub: boolean
  is_dubbed: boolean
}

export interface ImdbMetaData {
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

export interface VideoMetaData {
  name: string
  subtitle_path?: string // PathBuf as string
  year?: number
  files_data: VideoFileData[]
  series?: SeriesMeta
  imdb_metadata?: ImdbMetaData
}
