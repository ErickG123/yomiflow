export type KanaAlphabetSelection = 'hiragana' | 'katakana' | 'both'
export type KanaVariationCategory = 'base' | 'dakuon' | 'yoon'

export interface KanaCharacter {
  character: string
  romaji: string
  alphabet: 'hiragana' | 'katakana'
  category: KanaVariationCategory
}

export interface KanaMasteryRecord {
  character: string
  romaji: string
  kana_type: string
  alphabet: string
  category: string
  correct_count: number
  wrong_count: number
  total_attempts: number
  correct_attempts: number
  accuracy_percentage: number
  last_practiced: number
}

export interface KanaProficiencyStats {
  hiragana_proficiency: number
  katakana_proficiency: number
  overall_proficiency: number
  total_attempts: number
  total_correct: number
}

export interface KanaQuizConfig {
  alphabet: KanaAlphabetSelection
  includeBase: boolean
  includeDakuon: boolean
  includeYoon: boolean
}
