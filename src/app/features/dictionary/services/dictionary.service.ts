import { Injectable, inject, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { DictionaryEntry, ImportResult, KanjiEntry } from '../models/dictionary.model'
import { AppStateService } from '../../../core/services/app-state.service'

@Injectable({
  providedIn: 'root',
})
export class DictionaryService {
  private readonly appState = inject(AppStateService)

  readonly searchResults = signal<DictionaryEntry[]>([])
  readonly kanjiSearchResults = signal<KanjiEntry[]>([])
  readonly isLoading = signal<boolean>(false)
  readonly addedCardIds = signal<number[]>([])
  readonly isImporting = signal<boolean>(false)

  async searchDictionary(query: string): Promise<DictionaryEntry[]> {
    return invoke<DictionaryEntry[]>('search_dictionary', { query })
  }

  async searchKanjis(query: string): Promise<KanjiEntry[]> {
    return invoke<KanjiEntry[]>('search_kanjis', { query })
  }

  async search(query: string): Promise<{ dictResults: DictionaryEntry[]; kanjiResults: KanjiEntry[] }> {
    const trimmed = query.trim()
    if (!trimmed) {
      this.searchResults.set([])
      this.kanjiSearchResults.set([])
      return { dictResults: [], kanjiResults: [] }
    }

    this.isLoading.set(true)
    try {
      const [dictResults, kanjiResults] = await Promise.all([
        this.searchDictionary(trimmed).catch((err) => {
          console.error('Failed to search dictionary:', err)
          return [] as DictionaryEntry[]
        }),
        this.searchKanjis(trimmed).catch((err) => {
          console.error('Failed to search kanjis:', err)
          return [] as KanjiEntry[]
        }),
      ])

      this.searchResults.set(dictResults)
      this.kanjiSearchResults.set(kanjiResults)
      return { dictResults, kanjiResults }
    } finally {
      this.isLoading.set(false)
    }
  }

  async addToSrs(entryId: number, deckId?: string | null): Promise<void> {
    if (this.addedCardIds().includes(entryId)) {
      return
    }

    try {
      await invoke('add_to_srs', { entryId, deckId: deckId || null })
      this.addedCardIds.update((current) => [...current, entryId])
      this.searchResults.update((entries) =>
        entries.map((entry) => (entry.id === entryId ? { ...entry, in_srs: true } : entry))
      )
    } catch (error) {
      console.error('Failed to add card to SRS:', error)
      throw error
    }
  }

  async addCustomWordToSrs(
    kanji: string | null,
    reading: string,
    meaning: string,
    deckId?: string | null
  ): Promise<number> {
    try {
      const entryId = await invoke<number>('add_custom_word_to_srs', {
        kanji: kanji || null,
        reading,
        meaning,
        deckId: deckId || null,
      })
      this.addedCardIds.update((current) => [...current, entryId])
      return entryId
    } catch (error) {
      console.error('Failed to add custom word to SRS:', error)
      throw error
    }
  }

  async removeFromSrs(entryId: number): Promise<void> {
    try {
      await invoke('remove_from_srs', { entryId })
      this.addedCardIds.update((current) => current.filter((id) => id !== entryId))
      this.searchResults.update((entries) =>
        entries.map((entry) => (entry.id === entryId ? { ...entry, in_srs: false } : entry))
      )
    } catch (error) {
      console.error('Failed to remove card from SRS:', error)
      throw error
    }
  }

  async getKanjiBreakdown(word: string): Promise<KanjiEntry[]> {
    try {
      return await invoke<KanjiEntry[]>('get_kanji_breakdown', { word })
    } catch (error) {
      console.error('Failed to get kanji breakdown:', error)
      return []
    }
  }

  async openImportDialog(): Promise<void> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select Yomichan Dictionary Folder',
      })

      if (!selected) {
        return
      }

      const folderPath = typeof selected === 'string' ? selected : selected[0]
      if (!folderPath) {
        return
      }

      this.isImporting.set(true)
      this.appState.isProcessing.set(true)
      this.appState.progressMessage.set('A iniciar importação...')
      try {
        const res = await invoke<ImportResult>('import_dictionary', { folderPath })
        console.log('Import completed successfully:', res)
        alert(
          `Importação concluída com sucesso!\n\n` +
            `• Termos: ${res.terms_imported}\n` +
            `• Kanjis: ${res.kanjis_imported}\n` +
            `• Tags: ${res.tags_imported}\n` +
            `• Total: ${res.total_imported} registros adicionados.`
        )
      } finally {
        this.appState.isProcessing.set(false)
      }
    } catch (error) {
      console.error('Failed to import dictionary:', error)
      alert(`Error importing dictionary: ${error}`)
    } finally {
      this.isImporting.set(false)
    }
  }

  async openImportJlptKanjiDialog(): Promise<void> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Selecionar Pasta ou Ficheiro JSON de Kanjis (JLPT)',
      })

      if (!selected) {
        return
      }

      const path = typeof selected === 'string' ? selected : selected[0]
      if (!path) {
        return
      }

      this.isImporting.set(true)
      this.appState.isProcessing.set(true)
      this.appState.progressMessage.set('A iniciar importação de Kanjis JLPT...')
      try {
        const count = await invoke<number>('import_jlpt_kanjis', { path })
        alert(`Importação de Kanjis concluída com sucesso!\n\n• ${count} Kanjis foram importados para o banco de dados.`)
      } finally {
        this.appState.isProcessing.set(false)
      }
    } catch (error) {
      console.error('Failed to import JLPT kanjis:', error)
      alert(`Erro ao importar Kanjis JLPT: ${error}`)
    } finally {
      this.isImporting.set(false)
    }
  }

  async openImportKanjiVgDialog(): Promise<void> {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Selecionar Pasta de ficheiros SVG do KanjiVG',
      })

      if (!selected) {
        return
      }

      const folderPath = typeof selected === 'string' ? selected : selected[0]
      if (!folderPath) {
        return
      }

      this.isImporting.set(true)
      this.appState.isProcessing.set(true)
      this.appState.progressMessage.set('A importar traçados SVG do KanjiVG...')
      try {
        const count = await invoke<number>('import_kanjivg_folder', { folderPath })
        alert(`Importação do KanjiVG concluída com sucesso!\n\n• ${count} traçados de Kanjis foram importados.`)
      } finally {
        this.appState.isProcessing.set(false)
      }
    } catch (error) {
      console.error('Failed to import KanjiVG folder:', error)
      alert(`Erro ao importar pasta do KanjiVG: ${error}`)
    } finally {
      this.isImporting.set(false)
    }
  }

  async getKanjiSvg(kanji: string): Promise<string | null> {
    return invoke<string | null>('get_kanji_svg', { kanji })
  }
}
