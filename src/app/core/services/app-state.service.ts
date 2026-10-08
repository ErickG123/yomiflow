import { Injectable, signal } from '@angular/core'
import { listen } from '@tauri-apps/api/event'

@Injectable({
  providedIn: 'root',
})
export class AppStateService {
  readonly isProcessing = signal<boolean>(false)
  readonly progressMessage = signal<string>('')

  async initTauriListeners(): Promise<void> {
    try {
      await listen<string>('backend-progress', (event) => {
        this.progressMessage.set(event.payload)
      })
    } catch (error) {
      console.warn('Failed to register backend-progress Tauri listener:', error)
    }
  }
}
