import { Injectable, signal } from '@angular/core'

@Injectable({ providedIn: 'root' })
export class PomodoroControlService {
  private readonly startRequestedState = signal(false)
  readonly startRequested = this.startRequestedState.asReadonly()

  requestStart(): void {
    this.startRequestedState.set(true)
  }

  consumeStartRequest(): void {
    this.startRequestedState.set(false)
  }
}
