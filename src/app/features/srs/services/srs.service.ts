import { Injectable, computed, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { ReviewCard } from '../models/review-card.model'

@Injectable({
  providedIn: 'root',
})
export class SrsService {
  readonly dueCards = signal<ReviewCard[]>([])
  readonly isLoading = signal<boolean>(false)

  readonly currentCard = computed(() => {
    const list = this.dueCards()
    return list.length > 0 ? list[0] : null
  })

  async getDueCards(): Promise<ReviewCard[]> {
    this.isLoading.set(true)
    try {
      const cards = await invoke<ReviewCard[]>('get_due_cards')
      this.dueCards.set(cards)
      return cards
    } catch (error) {
      console.error('Failed to get due cards:', error)
      this.dueCards.set([])
      return []
    } finally {
      this.isLoading.set(false)
    }
  }

  async answerCurrentCard(rating: number): Promise<void> {
    const card = this.currentCard()
    if (!card) {
      return
    }

    try {
      await invoke('answer_card', {
        cardId: card.card_id,
        rating,
      })

      // Remove current card from queue to immediately reveal next card
      this.dueCards.update((list) => list.slice(1))
    } catch (error) {
      console.error('Failed to answer card:', error)
      throw error
    }
  }
}
