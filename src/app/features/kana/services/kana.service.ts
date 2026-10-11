import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import {
  KanaCharacter,
  KanaMasteryRecord,
  KanaProficiencyStats,
  KanaQuizConfig,
} from '../models/kana.model'

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
  readonly allCharacters = signal<KanaCharacter[]>([])

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

  async loadKanaCharacters(): Promise<KanaCharacter[]> {
    try {
      const items = await invoke<KanaCharacter[]>('get_kana_characters')
      this.allCharacters.set(items)
      return items
    } catch (err) {
      console.warn('Erro ao carregar caracteres de kana do backend:', err)
      return []
    }
  }

  async updateKanaScore(character: string, isCorrect: boolean): Promise<KanaMasteryRecord> {
    const record = await invoke<KanaMasteryRecord>('update_kana_score', {
      character,
      isCorrect,
    })
    void this.loadProficiency()
    void this.loadMasteryList()
    return record
  }

  getFilteredCharacters(config: KanaQuizConfig): KanaCharacter[] {
    const all = this.allCharacters()
    return all.filter((kana) => {
      // Alphabet check
      if (config.alphabet === 'hiragana' && kana.alphabet !== 'hiragana') return false
      if (config.alphabet === 'katakana' && kana.alphabet !== 'katakana') return false

      // Category check
      if (kana.category === 'base' && !config.includeBase) return false
      if (kana.category === 'dakuon' && !config.includeDakuon) return false
      if (kana.category === 'yoon' && !config.includeYoon) return false

      return true
    })
  }
}
