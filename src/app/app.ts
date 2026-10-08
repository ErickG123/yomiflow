import { Component, OnInit, inject, signal } from '@angular/core'
import { FormsModule } from '@angular/forms'
import { RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router'
import { TranslocoPipe, TranslocoService } from '@ngneat/transloco'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { NotificationState, SettingsService } from './core/services/settings.service'
import { SchedulerService } from './core/services/scheduler.service'

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive, TranslocoPipe, FormsModule],
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App implements OnInit {
  private readonly translocoService = inject(TranslocoService, { optional: true })
  readonly settingsService = inject(SettingsService)
  readonly schedulerService = inject(SchedulerService)

  readonly isSettingsOpen = signal<boolean>(false)

  readonly availableLangs = [
    { id: 'pt', label: 'PT' },
    { id: 'en', label: 'EN' },
    { id: 'es', label: 'ES' },
  ]

  readonly activeLang = signal<string>(this.translocoService?.getActiveLang() ?? 'en')

  async ngOnInit(): Promise<void> {
    try {
      let permissionGranted = await isPermissionGranted()
      if (!permissionGranted) {
        const permission = await requestPermission()
        permissionGranted = permission === 'granted'
      }

      if (permissionGranted && this.settingsService.notificationState() === 'on') {
        sendNotification({
          title: 'YomiFlow',
          body: 'O YomiFlow está a correr em background!',
        })
      }
    } catch (error) {
      console.warn('Tauri notification plugin not active or supported in this context:', error)
    }
  }

  changeLanguage(lang: string): void {
    this.translocoService?.setActiveLang(lang)
    this.activeLang.set(lang)
  }

  setNotificationState(state: NotificationState): void {
    this.settingsService.setNotificationState(state)
  }


  toggleSettings(): void {
    this.isSettingsOpen.update((open) => !open)
  }

  closeSettings(): void {
    this.isSettingsOpen.set(false)
  }

  exitApp(): void {
    this.settingsService.hardExit()
  }
}
