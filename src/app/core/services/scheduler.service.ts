import { Injectable, effect, inject } from '@angular/core'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { DashboardService } from '../../features/dashboard/services/dashboard.service'
import { SettingsService } from './settings.service'

@Injectable({
  providedIn: 'root',
})
export class SchedulerService {
  private readonly settingsService = inject(SettingsService)
  private readonly dashboardService = inject(DashboardService)

  private intervalId: ReturnType<typeof setInterval> | null = null

  constructor() {
    effect(() => {
      const minutes = this.settingsService.notificationInterval()
      this.setupTimer(minutes)
    })
  }

  private setupTimer(minutes: number): void {
    if (this.intervalId) {
      clearInterval(this.intervalId)
      this.intervalId = null
    }

    const ms = minutes * 60 * 1000

    this.intervalId = setInterval(async () => {
      await this.checkAndNotify()
    }, ms)
  }

  private async checkAndNotify(): Promise<void> {
    const state = this.settingsService.notificationState()
    if (state === 'off') {
      return
    }

    if (state === 'paused') {
      if (!this.settingsService.isPauseExpired()) {
        return
      }
      await this.settingsService.setNotificationState('on')
    }

    try {
      const stats = await this.dashboardService.getStats()
      const pendingReviews = stats.due_today

      if (pendingReviews > 0) {
        let granted = await isPermissionGranted()
        if (!granted) {
          const permission = await requestPermission()
          granted = permission === 'granted'
        }

        if (granted) {
          sendNotification({
            title: 'YomiFlow - Hora de Estudar!',
            body: `Você tem ${pendingReviews} cartão(ões) aguardando revisão.`,
          })
        }
      }
    } catch (error) {
      console.warn('Failed to check due cards for scheduler notification:', error)
    }
  }
}
