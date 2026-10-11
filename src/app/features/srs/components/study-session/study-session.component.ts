import { ChangeDetectionStrategy, Component, HostListener, OnInit, inject, signal } from '@angular/core'
import { ActivatedRoute, RouterLink } from '@angular/router'
import { TranslocoPipe } from '@ngneat/transloco'
import { SrsService } from '../../services/srs.service'
import { AudioService } from '../../../../core/services/audio.service'
import { KanjiPadComponent } from '../../../../shared/components/kanji-pad/kanji-pad.component'
import { PitchAccentComponent } from '../../../../shared/components/pitch-accent/pitch-accent.component'
import { KanjiBreakdownComponent } from '../../../../shared/components/kanji-breakdown/kanji-breakdown.component'

@Component({
  selector: 'app-study-session',
  standalone: true,
  imports: [RouterLink, TranslocoPipe, KanjiPadComponent, PitchAccentComponent, KanjiBreakdownComponent],
  templateUrl: './study-session.component.html',
  styleUrl: './study-session.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class StudySessionComponent implements OnInit {
  private readonly srsService = inject(SrsService)
  private readonly route = inject(ActivatedRoute)
  readonly audioService = inject(AudioService)

  readonly dueCards = this.srsService.dueCards
  readonly isLoading = signal<boolean>(true)
  readonly currentCard = this.srsService.currentCard
  readonly isFlipped = signal<boolean>(false)

  // Japanese Kanji Unicode block matcher (\u4e00-\u9faf)
  readonly kanjiRegex = /[\u4e00-\u9faf]/

  async ngOnInit(): Promise<void> {
    try {
      const deckId = this.route.snapshot.queryParamMap.get('deckId')
      await this.srsService.getDueCards(deckId)
    } finally {
      this.isLoading.set(false)
    }
  }

  flipCard(): void {
    this.isFlipped.update((flipped) => !flipped)
  }

  async rateCard(rating: number): Promise<void> {
    try {
      await this.srsService.answerCurrentCard(rating)
      this.isFlipped.set(false)
    } catch (error) {
      console.error('Error answering card:', error)
    }
  }

  speakCurrentWord(): void {
    const card = this.currentCard()
    if (!card) {
      return
    }
    // Prefer the reading or kanji pronunciation
    this.audioService.speak(card.reading || card.kanji || '')
  }

  hasKanji(cardText: string | null | undefined): boolean {
    if (!cardText) {
      return false
    }
    return this.kanjiRegex.test(cardText)
  }

  getKanjiCharacters(cardText: string | null | undefined): string[] {
    if (!cardText) {
      return []
    }
    const matches = cardText.match(/[\u4e00-\u9faf]/g)
    return matches ? Array.from(new Set(matches)) : []
  }

  @HostListener('window:keydown', ['$event'])
  handleKeyboardEvent(event: KeyboardEvent): void {
    const activeTag = document.activeElement?.tagName.toLowerCase()
    if (activeTag === 'input' || activeTag === 'textarea') {
      return
    }

    if (!this.currentCard()) {
      return
    }

    // Audio Playback Hotkeys: 'r' (Read) or 'p' (Pronounce)
    if (event.key === 'r' || event.key === 'R' || event.key === 'p' || event.key === 'P') {
      event.preventDefault()
      this.speakCurrentWord()
      return
    }

    if (event.code === 'Space') {
      event.preventDefault()
      if (!this.isFlipped()) {
        this.flipCard()
      }
    } else if (this.isFlipped()) {
      if (event.key === '1') {
        this.rateCard(1)
      } else if (event.key === '2') {
        this.rateCard(2)
      } else if (event.key === '3') {
        this.rateCard(3)
      } else if (event.key === '4') {
        this.rateCard(4)
      }
    }
  }
}
