import { Component, OnInit, inject, signal } from '@angular/core'
import { FormsModule } from '@angular/forms'
import { NavigationEnd, Router, RouterLink, RouterLinkActive, RouterOutlet } from '@angular/router'
import { filter } from 'rxjs'
import { TranslocoDirective, TranslocoPipe, TranslocoService } from '@ngneat/transloco'
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification'
import { invoke } from '@tauri-apps/api/core'
import { SettingsService } from './core/services/settings.service'
import { NotificationState, SettingsTab } from './core/models/settings.model'
import { SchedulerService } from './core/services/scheduler.service'
import { AppStateService } from './core/services/app-state.service'
import { DictionaryService } from './features/dictionary/services/dictionary.service'
import { AiModelService } from './features/reader/services/ai-model.service'
import { CloudSyncComponent } from './features/settings/components/cloud-sync/cloud-sync.component'

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, RouterLink, RouterLinkActive, TranslocoDirective, TranslocoPipe, FormsModule, CloudSyncComponent],
  templateUrl: './app.html',
  styleUrl: './app.scss',
})
export class App implements OnInit {
  private readonly router = inject(Router, { optional: true })
  private readonly translocoService = inject(TranslocoService, { optional: true })
  readonly settingsService = inject(SettingsService)
  readonly schedulerService = inject(SchedulerService)
  readonly appState = inject(AppStateService)
  readonly dictionaryService = inject(DictionaryService)
  readonly aiModelService = inject(AiModelService)

  readonly isSettingsOpen = signal<boolean>(false)
  readonly settingsActiveTab = signal<SettingsTab>('general')
  readonly isAiReady = signal<boolean>(false)
  readonly settingsToast = signal<string | null>(null)
  readonly isOnboardingRoute = signal<boolean>(false)

  readonly availableLangs = [
    { id: 'pt', label: 'PT' },
    { id: 'en', label: 'EN' },
    { id: 'es', label: 'ES' },
  ]

  readonly activeLang = signal<string>(this.translocoService?.getActiveLang() ?? 'en')

  async ngOnInit(): Promise<void> {
    if (this.router) {
      this.isOnboardingRoute.set(this.router.url.includes('/onboarding'))
      this.router.events
        .pipe(filter((event): event is NavigationEnd => event instanceof NavigationEnd))
        .subscribe((event) => {
          this.isOnboardingRoute.set(event.urlAfterRedirects.includes('/onboarding'))
        })
    }

    await this.checkInitialOnboarding()
    await this.appState.initTauriListeners()
    await this.refreshAiStatus()

    try {
      let permissionGranted = await isPermissionGranted()
      if (!permissionGranted) {
        const permission = await requestPermission()
        permissionGranted = permission === 'granted'
      }

      if (permissionGranted && this.settingsService.notificationState() === 'on') {
        const bodyMsg =
          this.translocoService?.translate('nav.bg_notification') ??
          'O YomiFlow está a correr em background!'
        sendNotification({
          title: 'YomiFlow',
          body: bodyMsg,
        })
      }
    } catch (error) {
      console.warn('Tauri notification plugin not active or supported in this context:', error)
    }
  }

  private async checkInitialOnboarding(): Promise<void> {
    if (!this.router) return

    try {
      const isCompleted = await invoke<boolean>('has_completed_onboarding')
      const currentUrl = this.router.url

      if (isCompleted) {
        if (currentUrl === '/' || currentUrl === '/onboarding') {
          await this.router.navigate(['/dashboard'])
        }
      } else {
        if (currentUrl !== '/onboarding') {
          await this.router.navigate(['/onboarding'])
        }
      }
    } catch (err) {
      console.warn('Could not check onboarding status on startup:', err)
    }
  }

  changeLanguage(lang: string): void {
    this.translocoService?.setActiveLang(lang)
    this.activeLang.set(lang)
  }

  setNotificationState(state: NotificationState): void {
    this.settingsService.setNotificationState(state)
  }


  async toggleSettings(): Promise<void> {
    const nextState = !this.isSettingsOpen()
    this.isSettingsOpen.set(nextState)
    if (nextState) {
      await this.refreshAiStatus()
    }
  }

  closeSettings(): void {
    this.isSettingsOpen.set(false)
  }

  async refreshAiStatus(): Promise<void> {
    try {
      const ready = await this.aiModelService.checkModelExists()
      this.isAiReady.set(ready)
    } catch {
      this.isAiReady.set(false)
    }
  }

  async downloadAiModel(): Promise<void> {
    const success = await this.aiModelService.ensureModelExists()
    if (success) {
      this.isAiReady.set(true)
      const toastText =
        this.translocoService?.translate('settings.ai_ready_toast') ??
        'Modelo MangaOCR pronto a usar!'
      this.showSettingsToast(toastText)
    }
  }

  private showSettingsToast(message: string): void {
    this.settingsToast.set(message)
    setTimeout(() => {
      this.settingsToast.set(null)
    }, 3500)
  }

  async clearDatabase(): Promise<void> {
    const confirmMsg =
      this.translocoService?.translate('settings.clear_db_confirm') ??
      'Tem a certeza? Isto irá apagar todas as palavras importadas e o seu progresso no SRS. Esta ação é irreversível.'
    if (window.confirm(confirmMsg)) {
      this.appState.isProcessing.set(true)
      this.appState.progressMessage.set('A iniciar...')
      try {
        await this.settingsService.clearDatabase()
        const successMsg =
          this.translocoService?.translate('settings.clear_db_success') ??
          'Base de dados limpa com sucesso!'
        alert(successMsg)
      } catch (error) {
        console.error('Failed to clear database:', error)
        alert('Erro ao limpar a base de dados: ' + error)
      } finally {
        this.appState.isProcessing.set(false)
      }
    }
  }

  exitApp(): void {
    this.settingsService.hardExit()
  }
}
