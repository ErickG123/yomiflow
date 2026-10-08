import { ChangeDetectionStrategy, Component, inject } from '@angular/core'
import { TranslocoPipe } from '@ngneat/transloco'
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

  readonly searchResults = this.dictionaryService.searchResults
  readonly isLoading = this.dictionaryService.isLoading
  readonly addedCardIds = this.dictionaryService.addedCardIds
  readonly isImporting = this.dictionaryService.isImporting

  onSearchInput(event: Event): void {
    const target = event.target as HTMLInputElement
    this.dictionaryService.search(target.value)
  }

  isAdded(entryId: number): boolean {
    return this.addedCardIds().includes(entryId)
  }

  onAddToSrs(entryId: number): void {
    this.dictionaryService.addToSrs(entryId)
  }

  onImportDictionary(): void {
    this.dictionaryService.openImportDialog()
  }
}
