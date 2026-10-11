export interface KanaCharacter {
  character: string
  romaji: string
  type: 'hiragana' | 'katakana'
  row: string // 'a', 'ka', 'sa', 'ta', 'na', 'ha', 'ma', 'ya', 'ra', 'wa'
}

export interface KanaMasteryRecord {
  character: string
  romaji: string
  kana_type: 'hiragana' | 'katakana'
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

export type KanaAlphabetFilter = 'hiragana' | 'katakana' | 'both'
