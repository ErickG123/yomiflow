import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core'
import { Router } from '@angular/router'
import { TranslocoDirective, TranslocoPipe, TranslocoService } from '@ngneat/transloco'
import { DeckService } from '../../services/deck.service'
import { DeckWithCounts } from '../../models/deck.model'

@Component({
  selector: 'app-deck-list',
  standalone: true,
  imports: [TranslocoDirective, TranslocoPipe],
  templateUrl: './deck-list.component.html',
  styleUrl: './deck-list.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DeckListComponent implements OnInit {
  readonly deckService = inject(DeckService)
  private readonly router = inject(Router)
  private readonly translocoService = inject(TranslocoService, { optional: true })

  readonly isCreateModalOpen = signal<boolean>(false)
  readonly newDeckName = signal<string>('')
  readonly newDeckDescription = signal<string>('')
  readonly isSubmitting = signal<boolean>(false)
  readonly editingDeck = signal<DeckWithCounts | null>(null)

  async ngOnInit(): Promise<void> {
    await this.deckService.loadDecks()
  }

  toggleCreateModal(): void {
    if (!this.isCreateModalOpen()) {
      this.openCreateModal()
      return
    }
    this.isCreateModalOpen.set(false)
    this.editingDeck.set(null)
    this.newDeckName.set('')
    this.newDeckDescription.set('')
  }

  openCreateModal(): void {
    this.editingDeck.set(null)
    this.newDeckName.set('')
    this.newDeckDescription.set('')
    this.isCreateModalOpen.set(true)
  }

  editDeck(deck: DeckWithCounts, event: Event): void {
    event.stopPropagation()
    this.editingDeck.set(deck)
    this.newDeckName.set(deck.name)
    this.newDeckDescription.set(deck.description ?? '')
    this.isCreateModalOpen.set(true)
  }

  onNameInput(event: Event): void {
    const target = event.target as HTMLInputElement
    this.newDeckName.set(target.value)
  }

  onDescInput(event: Event): void {
    const target = event.target as HTMLTextAreaElement
    this.newDeckDescription.set(target.value)
  }

  async saveDeck(): Promise<void> {
    const name = this.newDeckName().trim()
    if (!name) return

    this.isSubmitting.set(true)
    try {
      const description = this.newDeckDescription().trim()
      const editingDeck = this.editingDeck()
      if (editingDeck) {
        await this.deckService.updateDeck(editingDeck.id, name, description)
      } else {
        await this.deckService.createDeck(name, description)
      }
      this.toggleCreateModal()
    } catch (err) {
      console.error('Failed to create deck:', err)
      const errorKey = this.editingDeck() ? 'decks.update_deck_error' : 'decks.create_error'
      const errPrefix = this.translocoService?.translate(errorKey) ?? 'Erro ao salvar baralho:'
      alert(`${errPrefix} ${err}`)
    } finally {
      this.isSubmitting.set(false)
    }
  }

  async deleteDeck(id: string, event: Event): Promise<void> {
    event.stopPropagation()
    const confirmMsg =
      this.translocoService?.translate('decks.delete_confirm') ??
      'Deseja realmente excluir este baralho? Os cartões associados serão movidos para o baralho padrão.'
    if (!confirm(confirmMsg)) {
      return
    }

    try {
      await this.deckService.deleteDeck(id)
    } catch (err) {
      console.error('Failed to delete deck:', err)
      const errPrefix =
        this.translocoService?.translate('decks.delete_error') ?? 'Erro ao excluir baralho:'
      alert(`${errPrefix} ${err}`)
    }
  }

  studyDeck(deckId: string): void {
    this.deckService.activeDeckId.set(deckId)
    void this.router.navigate(['/study'], {
      queryParams: { deckId },
    })
  }
}
