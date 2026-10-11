import { ChangeDetectionStrategy, Component, OnInit, computed, inject, signal } from '@angular/core'
import { Router } from '@angular/router'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { TranslocoDirective, TranslocoService } from '@ngneat/transloco'
import { AiModelService } from '../../../reader/services/ai-model.service'
import { OnboardingStep } from '../../models/onboarding.model'

@Component({
  selector: 'app-onboarding',
  standalone: true,
  imports: [TranslocoDirective],
  templateUrl: './onboarding.component.html',
  styleUrl: './onboarding.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class OnboardingComponent implements OnInit {
  private readonly router = inject(Router)
  readonly aiService = inject(AiModelService)
  private readonly translocoService = inject(TranslocoService, { optional: true })

  readonly step = signal<OnboardingStep>('welcome')
  readonly isAiReady = signal<boolean>(false)
  readonly isImporting = signal<boolean>(false)
  readonly importedKanjiCount = signal<number>(0)

  readonly progressPercentage = computed<number>(() => {
    switch (this.step()) {
      case 'welcome':
        return 25
      case 'ai-model':
        return 50
      case 'kanjivg':
        return 75
      case 'completed':
        return 100
      default:
        return 0
    }
  })

  async ngOnInit(): Promise<void> {
    const aiExists = await this.aiService.checkModelExists()
    this.isAiReady.set(aiExists)
  }

  readonly currentStep = computed<number>(() => {
    switch (this.step()) {
      case 'welcome':
        return 1
      case 'ai-model':
        return 2
      case 'kanjivg':
        return 3
      case 'completed':
        return 4
      default:
        return 1
    }
  })

  previousStep(): void {
    const cur = this.currentStep()
    if (cur > 1) {
      if (cur === 2) this.step.set('welcome')
      else if (cur === 3) this.step.set('ai-model')
      else if (cur === 4) this.step.set('kanjivg')
    }
  }

  goToStep(target: OnboardingStep): void {
    this.step.set(target)
  }

  async downloadAiModel(): Promise<void> {
    const success = await this.aiService.ensureModelExists()
    if (success) {
      this.isAiReady.set(true)
      this.goToStep('kanjivg')
    }
  }

  skipAiModel(): void {
    this.goToStep('kanjivg')
  }

  async selectAndImportKanjiVgFolder(): Promise<void> {
    try {
      const dialogTitle =
        this.translocoService?.translate('onboarding.kanjivg_step.folder_dialog_title') ??
        'Selecionar pasta contendo ficheiros SVG do KanjiVG'
      const selected = await open({
        directory: true,
        multiple: false,
        title: dialogTitle,
      })

      if (!selected || typeof selected !== 'string') {
        return
      }

      this.isImporting.set(true)
      const count = await invoke<number>('import_kanjivg_folder', {
        folderPath: selected,
      })
      this.importedKanjiCount.set(count)
    } catch (err) {
      console.error('Erro ao importar pasta KanjiVG:', err)
      const alertPrefix =
        this.translocoService?.translate('onboarding.kanjivg_step.error_alert') ??
        'Falha ao importar KanjiVG:'
      alert(`${alertPrefix} ${err}`)
    } finally {
      this.isImporting.set(false)
    }
  }

  async finishOnboarding(): Promise<void> {
    try {
      await invoke('complete_onboarding')
    } catch (err) {
      console.error('Failed to mark onboarding as completed:', err)
    }
    await this.router.navigate(['/dashboard'])
  }
}
