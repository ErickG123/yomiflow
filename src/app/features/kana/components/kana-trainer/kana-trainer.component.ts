import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  OnDestroy,
  OnInit,
  ViewChild,
  inject,
  signal,
} from '@angular/core'
import { CommonModule } from '@angular/common'
import { FormsModule } from '@angular/forms'
import { TranslocoDirective, TranslocoService } from '@ngneat/transloco'
import { KanaService } from '../../services/kana.service'
import { KanaAlphabetFilter, KanaCharacter } from '../../models/kana.model'

@Component({
  selector: 'app-kana-trainer',
  standalone: true,
  imports: [CommonModule, FormsModule, TranslocoDirective],
  templateUrl: './kana-trainer.component.html',
  styleUrl: './kana-trainer.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class KanaTrainerComponent implements OnInit, OnDestroy {
  readonly kanaService = inject(KanaService)
  private readonly translocoService = inject(TranslocoService, { optional: true })

  @ViewChild('answerInput') answerInputRef?: ElementRef<HTMLInputElement>

  readonly filterMode = signal<KanaAlphabetFilter>('both')
  readonly currentKana = signal<KanaCharacter | null>(null)
  readonly userInput = signal<string>('')
  readonly streak = signal<number>(0)
  readonly sessionTotal = signal<number>(0)
  readonly sessionCorrect = signal<number>(0)
  readonly lastResult = signal<'correct' | 'wrong' | null>(null)
  readonly lastWrongAnswer = signal<{ char: string; expected: string } | null>(null)
  readonly responseTimes = signal<number[]>([])

  private questionStartTime: number = Date.now()
  private nextQuestionTimeout: ReturnType<typeof setTimeout> | null = null

  async ngOnInit(): Promise<void> {
    await this.kanaService.loadProficiency()
    await this.kanaService.loadMasteryList()
    this.nextQuestion()
  }

  ngOnDestroy(): void {
    if (this.nextQuestionTimeout) {
      clearTimeout(this.nextQuestionTimeout)
    }
  }

  setFilter(mode: KanaAlphabetFilter): void {
    this.filterMode.set(mode)
    this.streak.set(0)
    this.lastResult.set(null)
    this.lastWrongAnswer.set(null)
    this.nextQuestion()
  }

  nextQuestion(): void {
    if (this.nextQuestionTimeout) {
      clearTimeout(this.nextQuestionTimeout)
      this.nextQuestionTimeout = null
    }

    const pool = this.kanaService.getCharacters(this.filterMode())
    if (pool.length === 0) return

    // Avoid repeating exactly the same character in consecutive questions if pool > 1
    const current = this.currentKana()
    let candidates = pool
    if (pool.length > 1 && current) {
      candidates = pool.filter((k) => k.character !== current.character)
    }

    const randomIndex = Math.floor(Math.random() * candidates.length)
    this.currentKana.set(candidates[randomIndex])
    this.userInput.set('')
    this.lastResult.set(null)
    this.lastWrongAnswer.set(null)
    this.questionStartTime = Date.now()

    setTimeout(() => {
      this.answerInputRef?.nativeElement.focus()
    }, 50)
  }

  async checkAnswer(): Promise<void> {
    const current = this.currentKana()
    if (!current) return

    const elapsed = Date.now() - this.questionStartTime
    this.responseTimes.update((list) => [...list, elapsed])

    const trimmedInput = this.userInput().trim().toLowerCase()
    const isCorrect = trimmedInput === current.romaji.toLowerCase()

    this.sessionTotal.update((n) => n + 1)

    if (isCorrect) {
      this.sessionCorrect.update((n) => n + 1)
      this.streak.update((s) => s + 1)
      this.lastResult.set('correct')
      this.lastWrongAnswer.set(null)
    } else {
      this.streak.set(0)
      this.lastResult.set('wrong')
      this.lastWrongAnswer.set({
        char: current.character,
        expected: current.romaji,
      })
    }

    await this.kanaService.recordAttempt(
      current.character,
      current.romaji,
      current.type,
      isCorrect
    )

    if (isCorrect) {
      this.nextQuestionTimeout = setTimeout(() => {
        this.nextQuestion()
      }, 350)
    } else {
      // Allow user a moment to see the expected romaji before advancing
      this.nextQuestionTimeout = setTimeout(() => {
        this.nextQuestion()
      }, 1200)
    }
  }

  skipQuestion(): void {
    const current = this.currentKana()
    if (!current) return

    this.sessionTotal.update((n) => n + 1)
    this.streak.set(0)
    this.lastResult.set('wrong')
    this.lastWrongAnswer.set({
      char: current.character,
      expected: current.romaji,
    })

    void this.kanaService.recordAttempt(
      current.character,
      current.romaji,
      current.type,
      false
    )

    this.nextQuestionTimeout = setTimeout(() => {
      this.nextQuestion()
    }, 900)
  }

  onInputKeydown(event: KeyboardEvent): void {
    if (event.key === 'Enter') {
      event.preventDefault()
      void this.checkAnswer()
    } else if (event.key === 'Tab') {
      event.preventDefault()
      this.skipQuestion()
    }
  }

  getAccuracyPercentage(): number {
    const total = this.sessionTotal()
    if (total === 0) return 100
    return Math.round((this.sessionCorrect() / total) * 100)
  }

  getAverageSpeedSeconds(): string {
    const times = this.responseTimes()
    if (times.length === 0) return '0.0s'
    const avg = times.reduce((a, b) => a + b, 0) / times.length
    return `${(avg / 1000).toFixed(1)}s`
  }

  restartSession(): void {
    this.streak.set(0)
    this.sessionTotal.set(0)
    this.sessionCorrect.set(0)
    this.responseTimes.set([])
    this.lastResult.set(null)
    this.lastWrongAnswer.set(null)
    this.nextQuestion()
  }
}
