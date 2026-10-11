import { ChangeDetectionStrategy, Component, OnInit, computed, inject, signal } from '@angular/core'
import { Router } from '@angular/router'
import { open } from '@tauri-apps/plugin-dialog'
import { invoke } from '@tauri-apps/api/core'
import { AiModelService } from '../../../reader/services/ai-model.service'
import { OnboardingStep } from '../../models/onboarding.model'

@Component({
  selector: 'app-onboarding',
  standalone: true,
  templateUrl: './onboarding.component.html',
  styleUrl: './onboarding.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class OnboardingComponent implements OnInit {
  private readonly router = inject(Router)
  readonly aiService = inject(AiModelService)

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
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Selecionar pasta contendo ficheiros SVG do KanjiVG',
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
      alert(`Falha ao importar KanjiVG: ${err}`)
    } finally {
      this.isImporting.set(false)
    }
  }

  finishOnboarding(): void {
    this.router.navigate(['/dashboard'])
  }
}
