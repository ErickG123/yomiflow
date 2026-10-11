export interface JlptLevelCoverage {
  level: number
  level_name: string
  total_kanji: number
  learned_kanji: number
  kanji_percentage: number
  total_vocab: number
  learned_vocab: number
  vocab_percentage: number
  overall_percentage: number
}

export interface JlptAnalyticsStats {
  levels: JlptLevelCoverage[]
  total_kanji_learned: number
  total_vocab_learned: number
}
