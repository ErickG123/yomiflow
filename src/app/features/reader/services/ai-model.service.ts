import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { listen, UnlistenFn } from '@tauri-apps/api/event'

@Injectable({
  providedIn: 'root',
})
export class AiModelService {
  readonly isDownloading = signal<boolean>(false)
  readonly downloadProgress = signal<number>(0)
  readonly isModelReady = signal<boolean>(false)

  private unlistenProgress: UnlistenFn | null = null
  private unlistenComplete: UnlistenFn | null = null

  async checkModelExists(): Promise<boolean> {
    try {
      return await invoke<boolean>('check_model_exists')
    } catch (err) {
      console.warn('Failed to check AI model existence:', err)
      return false
    }
  }

  async ensureModelExists(): Promise<boolean> {
    const exists = await this.checkModelExists()
    if (exists) {
      this.isModelReady.set(true)
      return true
    }

    if (this.isDownloading()) {
      return false
    }

    this.isDownloading.set(true)
    this.downloadProgress.set(0)

    try {
      // Set up event listeners for real-time progress
      this.unlistenProgress = await listen<number>(
        'model-download-progress',
        (event) => {
          this.downloadProgress.set(event.payload)
        }
      )

      this.unlistenComplete = await listen(
        'model-download-complete',
        () => {
          this.isDownloading.set(false)
          this.isModelReady.set(true)
          this.downloadProgress.set(100)
        }
      )

      await invoke('download_ai_model')
      this.isModelReady.set(true)
      return true
    } catch (err) {
      console.error('Failed to download AI model:', err)
      alert(`Erro ao transferir modelo de IA: ${err}`)
      return false
    } finally {
      this.isDownloading.set(false)
      this.cleanupListeners()
    }
  }

  private cleanupListeners(): void {
    if (this.unlistenProgress) {
      this.unlistenProgress()
      this.unlistenProgress = null
    }
    if (this.unlistenComplete) {
      this.unlistenComplete()
      this.unlistenComplete = null
    }
  }
}
