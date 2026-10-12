export interface UnknownWordOccurrence {
  surface: string
  lemma: string
  reading: string
  start: number
  end: number
  retentionPercentage: number
}

export interface ReadabilityAnalysis {
  totalWords: number
  knownWords: number
  comprehensionPercentage: number
  averageRetentionPercentage: number
  unknownWords: UnknownWordOccurrence[]
}

export interface ReadabilityTextSegment {
  text: string
  isUnknown: boolean
}
