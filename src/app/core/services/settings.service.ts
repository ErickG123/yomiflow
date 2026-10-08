import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { exit } from '@tauri-apps/plugin-process'

export type NotificationState = 'on' | 'off' | 'paused'

@Injectable({
  providedIn: 'root',
})
export class SettingsService {
  private readonly STATE_KEY = 'notification_state'
  private readonly PAUSE_EXPIRY_KEY = 'notification_pause_expiry'
  private readonly INTERVAL_KEY = 'notification_interval'

  readonly notificationState = signal<NotificationState>('on')
  readonly notificationInterval = signal<number>(60)

  private pauseExpiry: number = 0

  constructor() {
    void this.loadSettings()
  }

  async loadSettings(): Promise<void> {
    try {
      const interval = await invoke<string | null>('get_setting', { key: this.INTERVAL_KEY })
      if (interval) {
        const parsed = parseInt(interval, 10)
        if (!isNaN(parsed) && parsed > 0) {
          this.notificationInterval.set(parsed)
        }
      }

      const expiryStr = await invoke<string | null>('get_setting', { key: this.PAUSE_EXPIRY_KEY })
      if (expiryStr) {
        this.pauseExpiry = Number(expiryStr) || 0
      }

      const state = await invoke<string | null>('get_setting', { key: this.STATE_KEY })
      if (state === 'paused') {
        if (Date.now() > this.pauseExpiry) {
          await this.setNotificationState('on')
        } else {
          this.notificationState.set('paused')
        }
      } else if (state === 'off' || state === 'on') {
        this.notificationState.set(state)
      }
    } catch (error) {
      console.warn('Failed to load settings from SQLite via Tauri IPC:', error)
    }
  }

  isPauseExpired(): boolean {
    return Date.now() > this.pauseExpiry
  }

  async setNotificationState(state: NotificationState): Promise<void> {
    this.notificationState.set(state)

    try {
      await invoke('set_setting', { key: this.STATE_KEY, value: state })

      if (state === 'paused') {
        this.pauseExpiry = Date.now() + 60 * 60 * 1000
        await invoke('set_setting', {
          key: this.PAUSE_EXPIRY_KEY,
          value: this.pauseExpiry.toString(),
        })
      } else {
        this.pauseExpiry = 0
        await invoke('set_setting', { key: this.PAUSE_EXPIRY_KEY, value: '0' })
      }
    } catch (error) {
      console.error('Failed to save notification state to SQLite:', error)
    }
  }

  async setNotificationInterval(minutes: number): Promise<void> {
    this.notificationInterval.set(minutes)

    try {
      await invoke('set_setting', {
        key: this.INTERVAL_KEY,
        value: minutes.toString(),
      })
    } catch (error) {
      console.error('Failed to save notification interval to SQLite:', error)
    }
  }

  async hardExit(): Promise<void> {
    try {
      await exit(0)
    } catch (error) {
      console.error('Failed to exit process:', error)
      window.close()
    }
  }
}
