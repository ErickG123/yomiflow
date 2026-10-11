import {
  ChangeDetectionStrategy,
  Component,
  OnInit,
  inject,
  signal,
} from '@angular/core'
import { Router } from '@angular/router'
import { DeckService } from '../../services/deck.service'

@Component({
  selector: 'app-deck-list',
  standalone: true,
  imports: [],
  templateUrl: './deck-list.component.html',
  styleUrl: './deck-list.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DeckListComponent implements OnInit {
  readonly deckService = inject(DeckService)
  private readonly router = inject(Router)

  readonly isCreateModalOpen = signal<boolean>(false)
  readonly newDeckName = signal<string>('')
  readonly newDeckDescription = signal<string>('')
  readonly isSubmitting = signal<boolean>(false)

  async ngOnInit(): Promise<void> {
    await this.deckService.loadDecks()
  }

  toggleCreateModal(): void {
    this.isCreateModalOpen.update((v) => !v)
    this.newDeckName.set('')
    this.newDeckDescription.set('')
  }

  onNameInput(event: Event): void {
    const target = event.target as HTMLInputElement
    this.newDeckName.set(target.value)
  }

  onDescInput(event: Event): void {
    const target = event.target as HTMLTextAreaElement
    this.newDeckDescription.set(target.value)
  }

  async createDeck(): Promise<void> {
    const name = this.newDeckName().trim()
    if (!name) return

    this.isSubmitting.set(true)
    try {
      await this.deckService.createDeck(name, this.newDeckDescription().trim())
      this.isCreateModalOpen.set(false)
      this.newDeckName.set('')
      this.newDeckDescription.set('')
    } catch (err) {
      console.error('Failed to create deck:', err)
      alert(`Erro ao criar baralho: ${err}`)
    } finally {
      this.isSubmitting.set(false)
    }
  }

  async deleteDeck(id: string, event: Event): Promise<void> {
    event.stopPropagation()
    if (!confirm('Deseja realmente excluir este baralho? Os cartões associados serão movidos para o baralho padrão.')) {
      return
    }

    try {
      await this.deckService.deleteDeck(id)
    } catch (err) {
      console.error('Failed to delete deck:', err)
      alert(`Erro ao excluir baralho: ${err}`)
    }
  }

  studyDeck(deckId: string): void {
    this.deckService.activeDeckId.set(deckId)
    void this.router.navigate(['/study'], {
      queryParams: { deckId },
    })
  }
}
