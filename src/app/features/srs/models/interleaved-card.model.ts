export interface VocabularyInterleavedCard {
  cardType: 'vocabulary'
  cardId: string
  deckId: string
  kanji: string | null
  reading: string
  meaning: string
  state: number
  pitch: string | null
}

export interface ImageInterleavedCard {
  cardType: 'image_occlusion'
  cardId: string
  deckId: string
}

export interface GrammarInterleavedCard {
  cardType: 'grammar'
  cardId: string
  deckId: string
  title: string
  contentMd: string
}

export interface KanaInterleavedCard {
  cardType: 'kana'
  character: string
  romaji: string
  alphabet: string
  category: string
}

export type InterleavedCard =
  | VocabularyInterleavedCard
  | ImageInterleavedCard
  | GrammarInterleavedCard
  | KanaInterleavedCard
