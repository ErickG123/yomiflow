import { Injectable, computed, inject, signal } from '@angular/core'
import { DictionaryEntry } from '../models/dictionary.model'
import { ContextualTooltipPosition } from '../models/contextual-tooltip.model'
import { DictionaryService } from './dictionary.service'

@Injectable({ providedIn: 'root' })
export class ContextualDictionaryService {
  private readonly dictionary = inject(DictionaryService)
  private requestVersion = 0

  readonly word = signal<string | null>(null)
  readonly entry = signal<DictionaryEntry | null>(null)
  readonly position = signal<ContextualTooltipPosition>({ x: 0, y: 0 })
  readonly isLoading = signal(false)
  readonly isVisible = computed(() => this.word() !== null)

  moveTo(x: number, y: number): void {
    this.position.set({ x, y })
  }

  async lookup(word: string, x: number, y: number): Promise<void> {
    const normalizedWord = word.trim()
    if (!normalizedWord) return

    const version = ++this.requestVersion
    this.word.set(normalizedWord)
    this.position.set({ x, y })
    this.entry.set(null)
    this.isLoading.set(true)

    try {
      const results = await this.dictionary.searchDictionary(normalizedWord)
      if (version === this.requestVersion) this.entry.set(results[0] ?? null)
    } catch {
      if (version === this.requestVersion) this.entry.set(null)
    } finally {
      if (version === this.requestVersion) this.isLoading.set(false)
    }
  }

  hide(): void {
    this.requestVersion++
    this.word.set(null)
    this.entry.set(null)
    this.isLoading.set(false)
  }
}
