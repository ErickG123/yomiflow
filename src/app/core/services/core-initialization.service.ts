import { Injectable, inject, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification'
import { TranslocoService } from '@ngneat/transloco'
import { firstValueFrom } from 'rxjs'
import { AppStateService } from './app-state.service'
import { SettingsService } from './settings.service'

@Injectable({ providedIn: 'root' })
export class CoreInitializationService {
  private readonly appState = inject(AppStateService)
  private readonly settings = inject(SettingsService)
  private readonly transloco = inject(TranslocoService)
  readonly onboardingComplete = signal<boolean | null>(null)
  private initialization?: Promise<void>

  initialize(): Promise<void> {
    this.initialization ??= this.initializeCore()
    return this.initialization
  }

  private async initializeCore(): Promise<void> {
    try {
      this.onboardingComplete.set(await invoke<boolean>('has_completed_onboarding'))
    } catch (error) {
      console.warn('Could not read onboarding state during core initialization:', error)
    }

    await this.appState.initTauriListeners()
    void this.initializeNotifications()
  }

  private async initializeNotifications(): Promise<void> {
    try {
      let permissionGranted = await isPermissionGranted()
      if (!permissionGranted) permissionGranted = (await requestPermission()) === 'granted'

      if (permissionGranted && this.settings.notificationState() === 'on') {
        const translatedBody = await firstValueFrom(
          this.transloco.selectTranslate('nav.bg_notification'),
        )
        await sendNotification({
          title: 'YomiFlow',
          body: translatedBody || 'YomiFlow is running in the background!',
        })
      }
    } catch (error) {
      console.warn('Tauri notifications are unavailable during core initialization:', error)
    }
  }
}
