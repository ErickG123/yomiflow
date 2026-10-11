import { Injectable, signal, computed } from '@angular/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { invoke } from '@tauri-apps/api/core'

import { PomodoroMode } from '../models/zen-mode.model'

@Injectable({
  providedIn: 'root',
})
export class ZenModeService {
  readonly isZenMode = signal<boolean>(false)

  // Pomodoro settings (seconds)
  readonly workDuration = 25 * 60 // 25 mins
  readonly breakDuration = 5 * 60 // 5 mins

  readonly currentMode = signal<PomodoroMode>('work')
  readonly timeLeft = signal<number>(this.workDuration)
  readonly isTimerRunning = signal<boolean>(false)

  private timerInterval: ReturnType<typeof setInterval> | null = null

  readonly formattedTime = computed(() => {
    const totalSecs = this.timeLeft()
    const mins = Math.floor(totalSecs / 60)
    const secs = totalSecs % 60
    return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
  })

  readonly progressPercentage = computed(() => {
    const total = this.currentMode() === 'work' ? this.workDuration : this.breakDuration
    const elapsed = total - this.timeLeft()
    return Math.min(100, Math.max(0, (elapsed / total) * 100))
  })

  async toggleZenMode(): Promise<void> {
    const next = !this.isZenMode()
    this.isZenMode.set(next)
    try {
      const appWindow = getCurrentWindow()
      await appWindow.setFullscreen(next)
    } catch (err) {
      console.warn('Could not set fullscreen mode via Tauri API:', err)
    }

    try {
      await invoke('set_discord_zen_presence', { enabled: next })
    } catch (err) {
      console.warn('Discord Rich Presence error:', err)
    }

    if (next && !this.isTimerRunning()) {
      this.startTimer()
    } else if (!next) {
      this.pauseTimer()
    }
  }

  async exitZenMode(): Promise<void> {
    this.isZenMode.set(false)
    this.pauseTimer()
    try {
      const appWindow = getCurrentWindow()
      await appWindow.setFullscreen(false)
    } catch (err) {
      console.warn('Could not set fullscreen mode via Tauri API:', err)
    }

    try {
      await invoke('set_discord_zen_presence', { enabled: false })
    } catch (err) {
      console.warn('Discord Rich Presence error:', err)
    }
  }

  startTimer(): void {
    if (this.isTimerRunning()) return
    this.isTimerRunning.set(true)
    this.timerInterval = setInterval(() => {
      this.tick()
    }, 1000)
  }

  pauseTimer(): void {
    this.isTimerRunning.set(false)
    if (this.timerInterval) {
      clearInterval(this.timerInterval)
      this.timerInterval = null
    }
  }

  resetTimer(): void {
    this.pauseTimer()
    const targetSec = this.currentMode() === 'work' ? this.workDuration : this.breakDuration
    this.timeLeft.set(targetSec)
  }

  switchMode(mode: PomodoroMode): void {
    this.pauseTimer()
    this.currentMode.set(mode)
    const targetSec = mode === 'work' ? this.workDuration : this.breakDuration
    this.timeLeft.set(targetSec)
  }

  private tick(): void {
    const current = this.timeLeft()
    if (current > 1) {
      this.timeLeft.set(current - 1)
    } else {
      // Switch mode automatically
      if (this.currentMode() === 'work') {
        this.switchMode('break')
      } else {
        this.switchMode('work')
      }
      this.startTimer()
    }
  }
}
