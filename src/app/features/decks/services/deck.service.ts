import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { Deck, DeckWithCounts, Tag } from '../models/deck.model'

@Injectable({
  providedIn: 'root',
})
export class DeckService {
  readonly decks = signal<DeckWithCounts[]>([])
  readonly tags = signal<Tag[]>([])
  readonly isLoading = signal<boolean>(false)
  readonly activeDeckId = signal<string | null>(null)
  readonly targetDeckId = signal<string>('default')

  async loadDecks(): Promise<DeckWithCounts[]> {
    this.isLoading.set(true)
    try {
      const data = await invoke<DeckWithCounts[]>('get_decks')
      this.decks.set(data)
      if (!data.some((deck) => deck.id === this.targetDeckId())) {
        this.targetDeckId.set(data[0]?.id ?? 'default')
      }
      return data
    } catch (error) {
      console.error('Failed to load decks:', error)
      this.decks.set([])
      return []
    } finally {
      this.isLoading.set(false)
    }
  }

  setTargetDeck(id: string): void {
    this.targetDeckId.set(id || 'default')
  }

  async loadTags(): Promise<Tag[]> {
    try {
      const data = await invoke<Tag[]>('get_tags')
      this.tags.set(data)
      return data
    } catch (error) {
      console.error('Failed to load tags:', error)
      return []
    }
  }

  async createDeck(name: string, description?: string): Promise<Deck> {
    try {
      const deck = await invoke<Deck>('create_deck', {
        name,
        description: description || null,
      })
      await this.loadDecks()
      return deck
    } catch (error) {
      console.error('Failed to create deck:', error)
      throw error
    }
  }

  async updateDeck(id: string, name: string, description?: string): Promise<void> {
    try {
      await invoke('update_deck', {
        id,
        name,
        description: description || null,
      })
      await this.loadDecks()
    } catch (error) {
      console.error('Failed to update deck:', error)
      throw error
    }
  }

  async deleteDeck(id: string): Promise<void> {
    try {
      await invoke('delete_deck', { id })
      await this.loadDecks()
    } catch (error) {
      console.error('Failed to delete deck:', error)
      throw error
    }
  }
}
