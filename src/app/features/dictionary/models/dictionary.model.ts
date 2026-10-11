export interface DictionaryEntry {
  id: number
  kanji: string | null
  reading: string
  meaning: string
  in_srs: boolean
  rank?: number | null
  pitch?: string | null
  tags?: string[]
}

export interface ImportResult {
  terms_imported: number
  kanjis_imported: number
  tags_imported: number
  total_imported: number
}

export interface KanjiEntry {
  character: string
  onyomi: string | null
  kunyomi: string | null
  meaning: string
  jlpt_level?: number | null
}
