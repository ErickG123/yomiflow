import { Component, effect, inject, signal } from '@angular/core'
import { FormsModule } from '@angular/forms'
import { TranslocoDirective, TranslocoService } from '@ngneat/transloco'
import { AppStateService } from '../../../core/services/app-state.service'
import { SettingsService } from '../../../core/services/settings.service'
import { NotificationState } from '../../../core/models/settings.model'
import { DictionaryService } from '../../../features/dictionary/services/dictionary.service'
import { AiModelService } from '../../../features/reader/services/ai-model.service'
import { CloudSyncComponent } from '../../../features/settings/components/cloud-sync/cloud-sync.component'
import { LayoutStateService } from '../../services/layout-state.service'

@Component({
  selector: 'app-settings-modal',
  standalone: true,
  imports: [FormsModule, TranslocoDirective, CloudSyncComponent],
  templateUrl: './settings-modal.component.html',
})
export class SettingsModalComponent {
  readonly layoutState = inject(LayoutStateService)
  readonly settingsService = inject(SettingsService)
  readonly appState = inject(AppStateService)
  readonly dictionaryService = inject(DictionaryService)
  readonly aiModelService = inject(AiModelService)
  private readonly translocoService = inject(TranslocoService)

  readonly isAiReady = signal(false)
  readonly settingsToast = signal<string | null>(null)
  readonly availableLangs = [
    { id: 'pt', label: 'PT' },
    { id: 'en', label: 'EN' },
    { id: 'es', label: 'ES' },
  ]
  readonly activeLang = signal(this.translocoService.getActiveLang() ?? 'en')
  private readonly refreshAiOnOpen = effect(() => {
    if (this.layoutState.isSettingsOpen()) void this.refreshAiStatus()
  })

  closeSettings(): void {
    this.layoutState.isSettingsOpen.set(false)
  }

  changeLanguage(lang: string): void {
    this.translocoService.setActiveLang(lang)
    this.activeLang.set(lang)
  }

  setNotificationState(state: NotificationState): void {
    this.settingsService.setNotificationState(state)
  }

  async refreshAiStatus(): Promise<void> {
    try {
      this.isAiReady.set(await this.aiModelService.checkModelExists())
    } catch {
      this.isAiReady.set(false)
    }
  }

  async downloadAiModel(): Promise<void> {
    if (!(await this.aiModelService.ensureModelExists())) return
    this.isAiReady.set(true)
    this.showSettingsToast(this.translocoService.translate('settings.ai_ready_toast'))
  }

  private showSettingsToast(message: string): void {
    this.settingsToast.set(message)
    setTimeout(() => this.settingsToast.set(null), 3500)
  }

  async clearDatabase(): Promise<void> {
    const confirmMessage = this.translocoService.translate('settings.clear_db_confirm')
    if (!window.confirm(confirmMessage)) return

    this.appState.isProcessing.set(true)
    this.appState.progressMessage.set('A iniciar...')
    try {
      await this.settingsService.clearDatabase()
      alert(this.translocoService.translate('settings.clear_db_success'))
    } catch (error) {
      console.error('Failed to clear database:', error)
      alert(`${this.translocoService.translate('common.error')}: ${error}`)
    } finally {
      this.appState.isProcessing.set(false)
    }
  }

  exitApp(): void {
    this.settingsService.hardExit()
  }
}
