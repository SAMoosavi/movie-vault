export interface MediaSearchResult {
  titles: SearchedMedia[]
}

export interface Image {
  url: string
}

export interface Rating {
  aggregateRating: number
}

export interface SearchedMedia {
  id: string
  type: string | number
  primaryTitle: string
  startYear: number
  originalTitle: string
  rating?: Rating
  primaryImage?: Image
}
