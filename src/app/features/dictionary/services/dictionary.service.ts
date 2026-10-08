import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { DictionaryEntry } from '../models/dictionary.model'

@Injectable({
  providedIn: 'root',
})
export class DictionaryService {
  readonly searchResults = signal<DictionaryEntry[]>([])
  readonly isLoading = signal<boolean>(false)
  readonly addedCardIds = signal<number[]>([])
  readonly isImporting = signal<boolean>(false)

  async search(query: string): Promise<DictionaryEntry[]> {
    if (!query.trim()) {
      this.searchResults.set([])
      return []
    }

    this.isLoading.set(true)
    try {
      const results = await invoke<DictionaryEntry[]>('search_dictionary', { query })
      this.searchResults.set(results)
      return results
    } catch (error) {
      console.error('Failed to search dictionary:', error)
      this.searchResults.set([])
      return []
    } finally {
      this.isLoading.set(false)
    }
  }

  async addToSrs(entryId: number): Promise<void> {
    if (this.addedCardIds().includes(entryId)) {
      return
    }

    try {
      await invoke('add_to_srs', { entryId })
      this.addedCardIds.update((current) => [...current, entryId])
      this.searchResults.update((entries) =>
        entries.map((entry) => (entry.id === entryId ? { ...entry, in_srs: true } : entry))
      )
    } catch (error) {
      console.error('Failed to add card to SRS:', error)
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
      const count = await invoke<number>('import_dictionary', { folderPath })
      console.log(`Successfully imported ${count} entries from ${folderPath}`)
      alert(`Import completed! ${count} words were added to your dictionary.`)
    } catch (error) {
      console.error('Failed to import dictionary:', error)
      alert(`Error importing dictionary: ${error}`)
    } finally {
      this.isImporting.set(false)
    }
  }
}
