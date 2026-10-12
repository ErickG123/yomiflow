import { Component, computed, signal } from '@angular/core'
import { DecimalPipe } from '@angular/common'
import { FormsModule } from '@angular/forms'
import { TranslocoPipe } from '@ngneat/transloco'
import { invoke } from '@tauri-apps/api/core'
import { ReadabilityAnalysis, ReadabilityTextSegment } from '../../models/readability.model'
import { DeckService } from '../../../decks/services/deck.service'

@Component({
  selector: 'app-readability-analyzer',
  standalone: true,
  imports: [DecimalPipe, FormsModule, TranslocoPipe],
  templateUrl: './readability-analyzer.component.html',
  styleUrl: './readability-analyzer.component.scss',
})
export class ReadabilityAnalyzerComponent {
  constructor(readonly deckService: DeckService) {
    void this.deckService.loadDecks()
  }

  readonly text = signal('')
  readonly analysis = signal<ReadabilityAnalysis | null>(null)
  readonly isAnalyzing = signal(false)
  readonly errorMessage = signal<string | null>(null)
  readonly isCardModalOpen = signal(false)
  readonly isCreatingCards = signal(false)
  readonly includeContext = signal(false)
  readonly selectedDeckId = signal('default')
  readonly cardError = signal<string | null>(null)
  readonly uniqueUnknownWords = computed(() => {
    const words = this.analysis()?.unknownWords ?? []
    const unique = new Map<string, (typeof words)[number]>()
    for (const word of words) {
      const key = `${word.lemma}\u0000${word.reading}`
      if (!unique.has(key)) unique.set(key, word)
    }
    return [...unique.values()]
  })
  readonly segments = computed<ReadabilityTextSegment[]>(() => {
    const analysis = this.analysis()
    const text = this.text()
    if (!analysis) return [{ text, isUnknown: false }]

    const result: ReadabilityTextSegment[] = []
    let cursor = 0
    for (const word of analysis.unknownWords) {
      const start = Math.max(cursor, Math.min(text.length, word.start))
      const end = Math.max(start, Math.min(text.length, word.end))
      if (start > cursor) result.push({ text: text.slice(cursor, start), isUnknown: false })
      if (end > start) result.push({ text: text.slice(start, end), isUnknown: true })
      cursor = end
    }
    if (cursor < text.length) result.push({ text: text.slice(cursor), isUnknown: false })
    return result
  })
  readonly ringDasharray = computed(() => {
    const percent = this.analysis()?.comprehensionPercentage ?? 0
    const circumference = 2 * Math.PI * 42
    return `${(percent / 100) * circumference} ${circumference}`
  })

  async analyze(): Promise<void> {
    if (!this.text().trim() || this.isAnalyzing()) return
    this.isAnalyzing.set(true)
    this.errorMessage.set(null)
    this.analysis.set(null)
    try {
      this.analysis.set(
        await invoke<ReadabilityAnalysis>('analyze_text_readability', { text: this.text() }),
      )
    } catch (error) {
      this.errorMessage.set(String(error))
    } finally {
      this.isAnalyzing.set(false)
    }
  }

  updateText(value: string): void {
    this.text.set(value)
    this.analysis.set(null)
    this.errorMessage.set(null)
  }

  openCardModal(): void {
    this.selectedDeckId.set(this.deckService.targetDeckId())
    this.cardError.set(null)
    this.isCardModalOpen.set(true)
  }

  async createCards(): Promise<void> {
    if (this.isCreatingCards()) return
    this.isCreatingCards.set(true)
    this.cardError.set(null)
    try {
      for (const word of this.uniqueUnknownWords()) {
        await invoke('add_readability_word_to_srs', {
          surface: word.surface,
          lemma: word.lemma,
          reading: word.reading,
          context: this.sentenceAt(word.start, word.end),
          includeContext: this.includeContext(),
          deckId: this.selectedDeckId(),
        })
      }
      await this.deckService.loadDecks()
      this.isCardModalOpen.set(false)
    } catch (error) {
      this.cardError.set(String(error))
    } finally {
      this.isCreatingCards.set(false)
    }
  }

  private sentenceAt(start: number, end: number): string {
    const text = this.text()
    const left =
      Math.max(
        text.lastIndexOf('。', start - 1),
        text.lastIndexOf('！', start - 1),
        text.lastIndexOf('？', start - 1),
        text.lastIndexOf('\n', start - 1),
      ) + 1
    const stops = ['。', '！', '？', '\n']
      .map((mark) => text.indexOf(mark, end))
      .filter((index) => index >= 0)
    const right = stops.length ? Math.min(...stops) + 1 : text.length
    return text.slice(left, right).trim() || text.slice(start, end)
  }
}
