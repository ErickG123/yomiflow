import { ChangeDetectionStrategy, Component, DestroyRef, inject, signal } from '@angular/core'
import { ActivatedRoute } from '@angular/router'
import { takeUntilDestroyed } from '@angular/core/rxjs-interop'
import { TranslocoPipe, TranslocoService } from '@ngneat/transloco'
import { DictionaryEntry, KanjiEntry } from '../../models/dictionary.model'
import { DictionaryService } from '../../services/dictionary.service'
import { AudioService } from '../../../../core/services/audio.service'
import { PitchAccentComponent } from '../../../../shared/components/pitch-accent/pitch-accent.component'
import { DeckService } from '../../../decks/services/deck.service'

@Component({
  selector: 'app-dictionary-search',
  standalone: true,
  imports: [TranslocoPipe, PitchAccentComponent],
  templateUrl: './dictionary-search.component.html',
  styleUrl: './dictionary-search.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DictionarySearchComponent {
  private readonly dictionaryService = inject(DictionaryService)
  private readonly route = inject(ActivatedRoute)
  private readonly destroyRef = inject(DestroyRef)
  private readonly translocoService = inject(TranslocoService, { optional: true })
  readonly deckService = inject(DeckService)
  readonly audioService = inject(AudioService)

  readonly searchResults = this.dictionaryService.searchResults
  readonly kanjiSearchResults = this.dictionaryService.kanjiSearchResults
  readonly isLoading = this.dictionaryService.isLoading
  readonly addedCardIds = this.dictionaryService.addedCardIds
  readonly isImporting = this.dictionaryService.isImporting
  readonly selectedDeckId = this.deckService.targetDeckId
  private currentQuery = ''

  readonly toastMessage = signal<string | null>(null)
  private toastTimeout: ReturnType<typeof setTimeout> | null = null

  // Map of entryId -> KanjiEntry[]
  readonly expandedKanjiMap = signal<Record<number, KanjiEntry[]>>({})
  readonly loadingKanjiSet = signal<Record<number, boolean>>({})

  constructor() {
    void this.deckService.loadDecks()
    this.route.queryParamMap.pipe(takeUntilDestroyed(this.destroyRef)).subscribe((params) => {
      const search = params.get('search')
      if (search) {
        this.currentQuery = search
        void this.dictionaryService.search(search, this.selectedDeckId())
      }
    })
  }

  onSearchInput(event: Event): void {
    const target = event.target as HTMLInputElement
    this.currentQuery = target.value
    void this.dictionaryService.search(target.value, this.selectedDeckId())
  }

  onTargetDeckChange(event: Event): void {
    this.deckService.setTargetDeck((event.target as HTMLSelectElement).value)
    if (this.currentQuery.trim()) {
      void this.dictionaryService.search(this.currentQuery, this.selectedDeckId())
    }
  }

  isAdded(entryId: number): boolean {
    return this.addedCardIds().includes(entryId)
  }

  isExpanded(entryId: number): boolean {
    return !!this.expandedKanjiMap()[entryId]
  }

  async toggleKanjiBreakdown(entry: DictionaryEntry): Promise<void> {
    if (this.isExpanded(entry.id)) {
      this.expandedKanjiMap.update((map) => {
        const copy = { ...map }
        delete copy[entry.id]
        return copy
      })
      return
    }

    const wordToLookup = entry.kanji ?? entry.reading
    this.loadingKanjiSet.update((set) => ({ ...set, [entry.id]: true }))
    try {
      const kanjis = await this.dictionaryService.getKanjiBreakdown(wordToLookup)
      this.expandedKanjiMap.update((map) => ({ ...map, [entry.id]: kanjis }))
    } finally {
      this.loadingKanjiSet.update((set) => ({ ...set, [entry.id]: false }))
    }
  }

  async toggleSrs(entry: DictionaryEntry): Promise<void> {
    try {
      if (entry.in_srs) {
        await this.dictionaryService.removeFromSrs(entry.id, this.selectedDeckId())
        this.showToast(
          this.translocoService?.translate('search.toast_removed') ?? 'Removido dos estudos',
        )
      } else {
        await this.dictionaryService.addToSrs(entry.id, this.selectedDeckId())
        this.showToast(
          this.translocoService?.translate('search.toast_added') ?? 'Adicionado aos estudos',
        )
      }
    } catch (error) {
      console.error('Failed to toggle SRS status:', error)
    }
  }

  private showToast(message: string): void {
    if (this.toastTimeout) {
      clearTimeout(this.toastTimeout)
    }
    this.toastMessage.set(message)
    this.toastTimeout = setTimeout(() => {
      this.toastMessage.set(null)
    }, 2500)
  }

  onImportDictionary(): void {
    this.dictionaryService.openImportDialog()
  }
}
