export interface OcclusionMask {
  x: number
  y: number
  width: number
  height: number
}

export interface ImageOcclusionCard {
  id: string
  deckId: string
  imageData: string
  masks: OcclusionMask[]
  createdAt: string
}

export interface ImageOcclusionSummary {
  id: string
  deckId: string
  maskCount: number
  createdAt: string
  state: number
}
