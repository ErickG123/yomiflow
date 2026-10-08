import { ChangeDetectionStrategy, Component, inject, signal } from '@angular/core'
import { TranslocoPipe, TranslocoService } from '@ngneat/transloco'
import { DictionaryEntry } from '../../models/dictionary.model'
import { DictionaryService } from '../../services/dictionary.service'

@Component({
  selector: 'app-dictionary-search',
  standalone: true,
  imports: [TranslocoPipe],
  templateUrl: './dictionary-search.component.html',
  styleUrl: './dictionary-search.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DictionarySearchComponent {
  private readonly dictionaryService = inject(DictionaryService)
  private readonly translocoService = inject(TranslocoService, { optional: true })

  readonly searchResults = this.dictionaryService.searchResults
  readonly isLoading = this.dictionaryService.isLoading
  readonly addedCardIds = this.dictionaryService.addedCardIds
  readonly isImporting = this.dictionaryService.isImporting

  readonly toastMessage = signal<string | null>(null)
  private toastTimeout: ReturnType<typeof setTimeout> | null = null

  onSearchInput(event: Event): void {
    const target = event.target as HTMLInputElement
    this.dictionaryService.search(target.value)
  }

  isAdded(entryId: number): boolean {
    return this.addedCardIds().includes(entryId)
  }

  async toggleSrs(entry: DictionaryEntry): Promise<void> {
    try {
      if (entry.in_srs) {
        await this.dictionaryService.removeFromSrs(entry.id)
        this.showToast(this.translocoService?.translate('search.toast_removed') ?? 'Removido dos estudos')
      } else {
        await this.dictionaryService.addToSrs(entry.id)
        this.showToast(this.translocoService?.translate('search.toast_added') ?? 'Adicionado aos estudos')
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
