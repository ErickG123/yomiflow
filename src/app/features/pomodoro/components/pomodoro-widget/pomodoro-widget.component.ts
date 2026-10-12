import { Component, OnDestroy, effect, inject, input, signal } from '@angular/core'
import { FormsModule } from '@angular/forms'
import { invoke } from '@tauri-apps/api/core'
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification'
import { TranslocoPipe, TranslocoService } from '@ngneat/transloco'
import { PomodoroPhase, PomodoroStatus, StudyCategory } from '../../models/pomodoro.model'
import { PomodoroControlService } from '../../services/pomodoro-control.service'

@Component({
  selector: 'app-pomodoro',
  standalone: true,
  imports: [FormsModule, TranslocoPipe],
  templateUrl: './pomodoro-widget.component.html',
  styleUrl: './pomodoro-widget.component.scss',
})
export class PomodoroWidgetComponent implements OnDestroy {
  readonly isCollapsed = input(false)
  private readonly pomodoroControl = inject(PomodoroControlService)
  private readonly transloco = inject(TranslocoService)
  readonly status = signal<PomodoroStatus>('idle')
  readonly phase = signal<PomodoroPhase>('focus')
  readonly focusMinutes = signal(25)
  readonly breakMinutes = signal(5)
  readonly category = signal<StudyCategory>('active_reading')
  readonly remainingSeconds = signal(25 * 60)
  private deadline = 0
  private intervalId?: ReturnType<typeof setInterval>
  private readonly trayStartEffect = effect(() => {
    if (this.pomodoroControl.startRequested()) {
      this.pomodoroControl.consumeStartRequest()
      this.start()
    }
  })

  get timeDisplay(): string {
    const remaining = this.remainingSeconds()
    return `${Math.floor(remaining / 60)
      .toString()
      .padStart(2, '0')}:${(remaining % 60).toString().padStart(2, '0')}`
  }

  ngOnDestroy(): void {
    this.stopTicker()
  }

  start(): void {
    if (this.status() === 'running') return
    const seconds = this.remainingSeconds() || this.phaseDurationSeconds()
    this.deadline = Date.now() + seconds * 1000
    this.status.set('running')
    this.stopTicker()
    this.intervalId = setInterval(() => void this.tick(), 250)
  }

  pause(): void {
    if (this.status() !== 'running') return
    this.remainingSeconds.set(Math.max(0, Math.ceil((this.deadline - Date.now()) / 1000)))
    this.status.set('paused')
    this.stopTicker()
  }

  reset(): void {
    this.stopTicker()
    this.phase.set('focus')
    this.status.set('idle')
    this.remainingSeconds.set(this.focusMinutes() * 60)
  }

  updateFocusMinutes(value: number): void {
    this.focusMinutes.set(value)
    if (this.status() === 'idle' && this.phase() === 'focus') this.remainingSeconds.set(value * 60)
  }

  updateBreakMinutes(value: number): void {
    this.breakMinutes.set(value)
    if (this.status() === 'idle' && this.phase() === 'break') this.remainingSeconds.set(value * 60)
  }

  private async tick(): Promise<void> {
    const remaining = Math.max(0, Math.ceil((this.deadline - Date.now()) / 1000))
    this.remainingSeconds.set(remaining)
    if (remaining > 0) return

    this.stopTicker()
    const finishedPhase = this.phase()
    if (finishedPhase === 'focus') {
      try {
        await invoke('save_study_session', {
          category: this.category(),
          durationMinutes: this.focusMinutes(),
        })
      } catch (error) {
        console.error('Failed to save completed study session:', error)
      }
    }

    await this.notify(finishedPhase)
    const nextPhase: PomodoroPhase = finishedPhase === 'focus' ? 'break' : 'focus'
    this.phase.set(nextPhase)
    this.remainingSeconds.set(this.phaseDurationSeconds())
    this.status.set('idle')
  }

  private async notify(finishedPhase: PomodoroPhase): Promise<void> {
    try {
      let granted = await isPermissionGranted()
      if (!granted) granted = (await requestPermission()) === 'granted'
      if (!granted) return
      await sendNotification({
        title: 'YomiFlow',
        body: this.transloco.translate(
          finishedPhase === 'focus'
            ? 'pomodoro.notifications.focus'
            : 'pomodoro.notifications.break',
        ),
      })
    } catch (error) {
      console.warn('Could not send Pomodoro notification:', error)
    }
  }

  private phaseDurationSeconds(): number {
    return (this.phase() === 'focus' ? this.focusMinutes() : this.breakMinutes()) * 60
  }

  private stopTicker(): void {
    if (this.intervalId) clearInterval(this.intervalId)
    this.intervalId = undefined
  }
}
