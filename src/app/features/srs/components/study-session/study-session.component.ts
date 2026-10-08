import { ChangeDetectionStrategy, Component, HostListener, OnInit, inject, signal } from '@angular/core'
import { RouterLink } from '@angular/router'
import { TranslocoPipe } from '@ngneat/transloco'
import { SrsService } from '../../services/srs.service'

@Component({
  selector: 'app-study-session',
  standalone: true,
  imports: [RouterLink, TranslocoPipe],
  templateUrl: './study-session.component.html',
  styleUrl: './study-session.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class StudySessionComponent implements OnInit {
  private readonly srsService = inject(SrsService)

  readonly dueCards = this.srsService.dueCards
  readonly isLoading = signal<boolean>(true)
  readonly currentCard = this.srsService.currentCard
  readonly isFlipped = signal<boolean>(false)

  async ngOnInit(): Promise<void> {
    try {
      await this.srsService.getDueCards()
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

  @HostListener('window:keydown', ['$event'])
  handleKeyboardEvent(event: KeyboardEvent): void {
    if (!this.currentCard()) {
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
