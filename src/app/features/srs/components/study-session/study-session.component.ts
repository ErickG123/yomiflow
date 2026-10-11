import { ChangeDetectionStrategy, Component, HostListener, computed, effect, inject, signal } from '@angular/core'
import { CommonModule } from '@angular/common'
import { FormsModule } from '@angular/forms'
import { RouterLink } from '@angular/router'
import { TranslocoPipe } from '@ngneat/transloco'
import { invoke } from '@tauri-apps/api/core'
import { AudioService } from '../../../../core/services/audio.service'
import { KanjiPadComponent } from '../../../../shared/components/kanji-pad/kanji-pad.component'
import { PitchAccentComponent } from '../../../../shared/components/pitch-accent/pitch-accent.component'
import { KanjiBreakdownComponent } from '../../../../shared/components/kanji-breakdown/kanji-breakdown.component'
import { ImageOcclusionCard } from '../../../image-occlusion/models/image-occlusion.model'
import { InterleavedCard, KanaInterleavedCard } from '../../models/interleaved-card.model'

@Component({
  selector: 'app-study-session',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterLink, TranslocoPipe, KanjiPadComponent, PitchAccentComponent, KanjiBreakdownComponent],
  templateUrl: './study-session.component.html',
  styleUrl: './study-session.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class StudySessionComponent {
  readonly audioService = inject(AudioService)
  readonly queue = signal<InterleavedCard[]>([])
  readonly currentCard = computed(() => this.queue()[0] ?? null)
  readonly isLoading = signal(true)
  readonly isFlipped = signal(false)
  readonly imageCard = signal<ImageOcclusionCard | null>(null)
  readonly revealedMasks = signal<number[]>([])
  readonly kanaInput = signal('')
  readonly kanaAnswered = signal(false)
  readonly kanaCorrect = signal(false)
  readonly isAnswering = signal(false)
  readonly answerError = signal<string | null>(null)
  private activeImageId: string | null = null

  readonly kanjiRegex = /[\u4e00-\u9faf]/

  constructor() {
    effect(() => {
      const card = this.currentCard()
      if (card?.cardType === 'image_occlusion') void this.loadImageCard(card.cardId)
      else {
        this.activeImageId = null
        this.imageCard.set(null)
      }
    })
    void this.loadQueue()
  }

  async loadQueue(): Promise<void> {
    this.isLoading.set(true)
    try {
      this.queue.set(await invoke<InterleavedCard[]>('get_interleaved_review_queue'))
    } catch (error) {
      this.answerError.set(String(error))
      this.queue.set([])
    } finally {
      this.isLoading.set(false)
    }
  }

  private async loadImageCard(cardId: string): Promise<void> {
    if (this.activeImageId === cardId) return
    this.activeImageId = cardId
    this.imageCard.set(null)
    try {
      const card = await invoke<ImageOcclusionCard>('get_image_occlusion_card', { cardId })
      const current = this.currentCard()
      if (current?.cardType === 'image_occlusion' && current.cardId === cardId) this.imageCard.set(card)
    } catch (error) {
      this.answerError.set(String(error))
    }
  }

  flipCard(): void {
    this.isFlipped.update((value) => !value)
  }

  async rateCard(rating: number): Promise<void> {
    const card = this.currentCard()
    if (!card || this.isAnswering()) return
    this.isAnswering.set(true)
    this.answerError.set(null)
    try {
      if (card.cardType === 'vocabulary') {
        await invoke('answer_card', { cardId: card.cardId, rating })
      } else if (card.cardType === 'image_occlusion') {
        await invoke('answer_image_occlusion_card', { cardId: card.cardId, rating })
      } else if (card.cardType === 'grammar') {
        await invoke('answer_grammar_review', { cardId: card.cardId, rating })
      } else {
        return
      }
      this.advance()
    } catch (error) {
      this.answerError.set(String(error))
    } finally {
      this.isAnswering.set(false)
    }
  }

  async submitKana(): Promise<void> {
    const card = this.currentCard()
    if (!card || card.cardType !== 'kana' || this.kanaAnswered() || this.isAnswering()) return
    const correct = this.kanaInput().trim().toLowerCase() === card.romaji.toLowerCase()
    this.isAnswering.set(true)
    this.answerError.set(null)
    try {
      await invoke('update_kana_score', { character: card.character, isCorrect: correct })
      this.kanaCorrect.set(correct)
      this.kanaAnswered.set(true)
    } catch (error) {
      this.answerError.set(String(error))
    } finally {
      this.isAnswering.set(false)
    }
  }

  revealMask(index: number): void {
    this.revealedMasks.update((indexes) => indexes.includes(index) ? indexes : [...indexes, index])
  }

  advance(): void {
    this.queue.update((cards) => cards.slice(1))
    this.isFlipped.set(false)
    this.revealedMasks.set([])
    this.kanaInput.set('')
    this.kanaAnswered.set(false)
    this.kanaCorrect.set(false)
  }

  speakCurrentWord(): void {
    const card = this.currentCard()
    if (card?.cardType === 'vocabulary') this.audioService.speak(card.reading || card.kanji || '')
  }

  hasKanji(text: string | null | undefined): boolean {
    return !!text && this.kanjiRegex.test(text)
  }

  getKanjiCharacters(text: string | null | undefined): string[] {
    const matches = text?.match(/[\u4e00-\u9faf]/g)
    return matches ? Array.from(new Set(matches)) : []
  }

  isKanaCard(card: InterleavedCard): card is KanaInterleavedCard {
    return card.cardType === 'kana'
  }

  @HostListener('window:keydown', ['$event'])
  handleKeyboardEvent(event: KeyboardEvent): void {
    const card = this.currentCard()
    const activeTag = document.activeElement?.tagName.toLowerCase()
    if (!card || activeTag === 'input' || activeTag === 'textarea') return
    if (card.cardType === 'vocabulary' && ['r', 'R', 'p', 'P'].includes(event.key)) {
      event.preventDefault()
      this.speakCurrentWord()
    } else if (event.code === 'Space' && (card.cardType === 'vocabulary' || card.cardType === 'grammar')) {
      event.preventDefault()
      this.flipCard()
    } else if (this.isFlipped() || (card.cardType === 'image_occlusion' && this.revealedMasks().length === (this.imageCard()?.masks.length ?? -1))) {
      const rating = Number(event.key)
      if (rating >= 1 && rating <= 4) this.rateCard(rating)
    }
  }
}
