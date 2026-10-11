export interface KanjiComponent {
  kanji: string
  component: string
  meaning: string
}

export interface KanjiDetails {
  kanji: string
  components: KanjiComponent[]
  mnemonic: string | null
}
