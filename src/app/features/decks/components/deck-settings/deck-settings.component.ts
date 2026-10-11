import {
  ChangeDetectionStrategy,
  Component,
  OnInit,
  inject,
  signal,
} from '@angular/core'
import { ActivatedRoute, Router } from '@angular/router'
import { DeckService } from '../../services/deck.service'

@Component({
  selector: 'app-deck-settings',
  standalone: true,
  imports: [],
  templateUrl: './deck-settings.component.html',
  styleUrl: './deck-settings.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DeckSettingsComponent implements OnInit {
  private readonly deckService = inject(DeckService)
  private readonly route = inject(ActivatedRoute)
  private readonly router = inject(Router)

  readonly deckId = signal<string>('')
  readonly deckName = signal<string>('')
  readonly deckDescription = signal<string>('')
  readonly isSaving = signal<boolean>(false)

  async ngOnInit(): Promise<void> {
    const id = this.route.snapshot.paramMap.get('id')
    if (!id) {
      void this.router.navigate(['/decks'])
      return
    }
    this.deckId.set(id)
    await this.deckService.loadDecks()
    const found = this.deckService.decks().find((d) => d.id === id)
    if (found) {
      this.deckName.set(found.name)
      this.deckDescription.set(found.description || '')
    }
  }

  onNameChange(event: Event): void {
    const target = event.target as HTMLInputElement
    this.deckName.set(target.value)
  }

  onDescChange(event: Event): void {
    const target = event.target as HTMLTextAreaElement
    this.deckDescription.set(target.value)
  }

  async saveSettings(): Promise<void> {
    const id = this.deckId()
    const name = this.deckName().trim()
    if (!id || !name) return

    this.isSaving.set(true)
    try {
      await this.deckService.updateDeck(id, name, this.deckDescription().trim())
      void this.router.navigate(['/decks'])
    } catch (err) {
      console.error('Failed to update deck:', err)
      alert(`Erro ao atualizar baralho: ${err}`)
    } finally {
      this.isSaving.set(false)
    }
  }
}
