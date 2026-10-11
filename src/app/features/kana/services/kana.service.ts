import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import {
  KanaAlphabetFilter,
  KanaCharacter,
  KanaMasteryRecord,
  KanaProficiencyStats,
} from '../models/kana.model'

export const HIRAGANA_DATA: KanaCharacter[] = [
  { character: 'あ', romaji: 'a', type: 'hiragana', row: 'a' },
  { character: 'い', romaji: 'i', type: 'hiragana', row: 'a' },
  { character: 'う', romaji: 'u', type: 'hiragana', row: 'a' },
  { character: 'え', romaji: 'e', type: 'hiragana', row: 'a' },
  { character: 'お', romaji: 'o', type: 'hiragana', row: 'a' },
  { character: 'か', romaji: 'ka', type: 'hiragana', row: 'ka' },
  { character: 'き', romaji: 'ki', type: 'hiragana', row: 'ka' },
  { character: 'く', romaji: 'ku', type: 'hiragana', row: 'ka' },
  { character: 'け', romaji: 'ke', type: 'hiragana', row: 'ka' },
  { character: 'こ', romaji: 'ko', type: 'hiragana', row: 'ka' },
  { character: 'さ', romaji: 'sa', type: 'hiragana', row: 'sa' },
  { character: 'し', romaji: 'shi', type: 'hiragana', row: 'sa' },
  { character: 'す', romaji: 'su', type: 'hiragana', row: 'sa' },
  { character: 'せ', romaji: 'se', type: 'hiragana', row: 'sa' },
  { character: 'そ', romaji: 'so', type: 'hiragana', row: 'sa' },
  { character: 'た', romaji: 'ta', type: 'hiragana', row: 'ta' },
  { character: 'ち', romaji: 'chi', type: 'hiragana', row: 'ta' },
  { character: 'つ', romaji: 'tsu', type: 'hiragana', row: 'ta' },
  { character: 'て', romaji: 'te', type: 'hiragana', row: 'ta' },
  { character: 'と', romaji: 'to', type: 'hiragana', row: 'ta' },
  { character: 'な', romaji: 'na', type: 'hiragana', row: 'na' },
  { character: 'に', romaji: 'ni', type: 'hiragana', row: 'na' },
  { character: 'ぬ', romaji: 'nu', type: 'hiragana', row: 'na' },
  { character: 'ね', romaji: 'ne', type: 'hiragana', row: 'na' },
  { character: 'の', romaji: 'no', type: 'hiragana', row: 'na' },
  { character: 'は', romaji: 'ha', type: 'hiragana', row: 'ha' },
  { character: 'ひ', romaji: 'hi', type: 'hiragana', row: 'ha' },
  { character: 'ふ', romaji: 'fu', type: 'hiragana', row: 'ha' },
  { character: 'へ', romaji: 'he', type: 'hiragana', row: 'ha' },
  { character: 'ほ', romaji: 'ho', type: 'hiragana', row: 'ha' },
  { character: 'ま', romaji: 'ma', type: 'hiragana', row: 'ma' },
  { character: 'み', romaji: 'mi', type: 'hiragana', row: 'ma' },
  { character: 'む', romaji: 'mu', type: 'hiragana', row: 'ma' },
  { character: 'め', romaji: 'me', type: 'hiragana', row: 'ma' },
  { character: 'も', romaji: 'mo', type: 'hiragana', row: 'ma' },
  { character: 'や', romaji: 'ya', type: 'hiragana', row: 'ya' },
  { character: 'ゆ', romaji: 'yu', type: 'hiragana', row: 'ya' },
  { character: 'よ', romaji: 'yo', type: 'hiragana', row: 'ya' },
  { character: 'ら', romaji: 'ra', type: 'hiragana', row: 'ra' },
  { character: 'り', romaji: 'ri', type: 'hiragana', row: 'ra' },
  { character: 'る', romaji: 'ru', type: 'hiragana', row: 'ra' },
  { character: 'れ', romaji: 're', type: 'hiragana', row: 'ra' },
  { character: 'ろ', romaji: 'ro', type: 'hiragana', row: 'ra' },
  { character: 'わ', romaji: 'wa', type: 'hiragana', row: 'wa' },
  { character: 'を', romaji: 'wo', type: 'hiragana', row: 'wa' },
  { character: 'ん', romaji: 'n', type: 'hiragana', row: 'wa' },
]

export const KATAKANA_DATA: KanaCharacter[] = [
  { character: 'ア', romaji: 'a', type: 'katakana', row: 'a' },
  { character: 'イ', romaji: 'i', type: 'katakana', row: 'a' },
  { character: 'ウ', romaji: 'u', type: 'katakana', row: 'a' },
  { character: 'エ', romaji: 'e', type: 'katakana', row: 'a' },
  { character: 'オ', romaji: 'o', type: 'katakana', row: 'a' },
  { character: 'カ', romaji: 'ka', type: 'katakana', row: 'ka' },
  { character: 'キ', romaji: 'ki', type: 'katakana', row: 'ka' },
  { character: 'ク', romaji: 'ku', type: 'katakana', row: 'ka' },
  { character: 'ケ', romaji: 'ke', type: 'katakana', row: 'ka' },
  { character: 'コ', romaji: 'ko', type: 'katakana', row: 'ka' },
  { character: 'サ', romaji: 'sa', type: 'katakana', row: 'sa' },
  { character: 'シ', romaji: 'shi', type: 'katakana', row: 'sa' },
  { character: 'ス', romaji: 'su', type: 'katakana', row: 'sa' },
  { character: 'セ', romaji: 'se', type: 'katakana', row: 'sa' },
  { character: 'ソ', romaji: 'so', type: 'katakana', row: 'sa' },
  { character: 'タ', romaji: 'ta', type: 'katakana', row: 'ta' },
  { character: 'チ', romaji: 'chi', type: 'katakana', row: 'ta' },
  { character: 'ツ', romaji: 'tsu', type: 'katakana', row: 'ta' },
  { character: 'テ', romaji: 'te', type: 'katakana', row: 'ta' },
  { character: 'ト', romaji: 'to', type: 'katakana', row: 'ta' },
  { character: 'ナ', romaji: 'na', type: 'katakana', row: 'na' },
  { character: 'ニ', romaji: 'ni', type: 'katakana', row: 'na' },
  { character: 'ヌ', romaji: 'nu', type: 'katakana', row: 'na' },
  { character: 'ネ', romaji: 'ne', type: 'katakana', row: 'na' },
  { character: 'ノ', romaji: 'no', type: 'katakana', row: 'na' },
  { character: 'ハ', romaji: 'ha', type: 'katakana', row: 'ha' },
  { character: 'ヒ', romaji: 'hi', type: 'katakana', row: 'ha' },
  { character: 'フ', romaji: 'fu', type: 'katakana', row: 'ha' },
  { character: 'ヘ', romaji: 'he', type: 'katakana', row: 'ha' },
  { character: 'ホ', romaji: 'ho', type: 'katakana', row: 'ha' },
  { character: 'マ', romaji: 'ma', type: 'katakana', row: 'ma' },
  { character: 'ミ', romaji: 'mi', type: 'katakana', row: 'ma' },
  { character: 'ム', romaji: 'mu', type: 'katakana', row: 'ma' },
  { character: 'メ', romaji: 'me', type: 'katakana', row: 'ma' },
  { character: 'モ', romaji: 'mo', type: 'katakana', row: 'ma' },
  { character: 'ヤ', romaji: 'ya', type: 'katakana', row: 'ya' },
  { character: 'ユ', romaji: 'yu', type: 'katakana', row: 'ya' },
  { character: 'ヨ', romaji: 'yo', type: 'katakana', row: 'ya' },
  { character: 'ラ', romaji: 'ra', type: 'katakana', row: 'ra' },
  { character: 'リ', romaji: 'ri', type: 'katakana', row: 'ra' },
  { character: 'ル', romaji: 'ru', type: 'katakana', row: 'ra' },
  { character: 'レ', romaji: 're', type: 'katakana', row: 'ra' },
  { character: 'ロ', romaji: 'ro', type: 'katakana', row: 'ra' },
  { character: 'ワ', romaji: 'wa', type: 'katakana', row: 'wa' },
  { character: 'ヲ', romaji: 'wo', type: 'katakana', row: 'wa' },
  { character: 'ン', romaji: 'n', type: 'katakana', row: 'wa' },
]

@Injectable({
  providedIn: 'root',
})
export class KanaService {
  readonly proficiency = signal<KanaProficiencyStats>({
    hiragana_proficiency: 0,
    katakana_proficiency: 0,
    overall_proficiency: 0,
    total_attempts: 0,
    total_correct: 0,
  })

  readonly masteryList = signal<KanaMasteryRecord[]>([])

  async loadProficiency(): Promise<KanaProficiencyStats> {
    try {
      const stats = await invoke<KanaProficiencyStats>('get_kana_proficiency_stats')
      this.proficiency.set(stats)
      return stats
    } catch (err) {
      console.warn('Erro ao carregar proficiência de kana:', err)
      return this.proficiency()
    }
  }

  async loadMasteryList(): Promise<KanaMasteryRecord[]> {
    try {
      const list = await invoke<KanaMasteryRecord[]>('get_kana_mastery_list')
      this.masteryList.set(list)
      return list
    } catch (err) {
      console.warn('Erro ao carregar lista de maestria de kana:', err)
      return []
    }
  }

  async recordAttempt(
    character: string,
    romaji: string,
    kanaType: 'hiragana' | 'katakana',
    isCorrect: boolean
  ): Promise<KanaMasteryRecord> {
    const record = await invoke<KanaMasteryRecord>('record_kana_attempt', {
      character,
      romaji,
      kanaType,
      isCorrect,
    })
    void this.loadProficiency()
    void this.loadMasteryList()
    return record
  }

  getCharacters(filter: KanaAlphabetFilter): KanaCharacter[] {
    if (filter === 'hiragana') return [...HIRAGANA_DATA]
    if (filter === 'katakana') return [...KATAKANA_DATA]
    return [...HIRAGANA_DATA, ...KATAKANA_DATA]
  }
}
