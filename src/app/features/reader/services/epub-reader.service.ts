import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

@Injectable({
  providedIn: 'root',
})
export class EpubReaderService {
  readonly currentEpubPath = signal<string | null>(null)
  readonly epubBase64Data = signal<string | null>(null)
  readonly isLoading = signal<boolean>(false)
  readonly errorMessage = signal<string | null>(null)

  async openEpubDialog(): Promise<string | null> {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'EPUB Light Novel', extensions: ['epub'] }],
      })

      if (!selected) {
        return null
      }

      const filePath = typeof selected === 'string' ? selected : selected[0]
      if (filePath) {
        await this.loadEpubFile(filePath)
        return filePath
      }
      return null
    } catch (err) {
      console.error('Failed to open EPUB file dialog:', err)
      return null
    }
  }

  async loadEpubFile(filePath: string): Promise<void> {
    this.isLoading.set(true)
    this.errorMessage.set(null)
    this.currentEpubPath.set(filePath)

    try {
      const dataUri = await invoke<string>('read_epub_file', { path: filePath })
      this.epubBase64Data.set(dataUri)
    } catch (err) {
      const msg = typeof err === 'string' ? err : 'Falha ao carregar ficheiro EPUB'
      this.errorMessage.set(msg)
      console.error('Error loading EPUB:', err)
    } finally {
      this.isLoading.set(false)
    }
  }

  async injectFurigana(htmlContent: string): Promise<string> {
    try {
      return await invoke<string>('inject_furigana_epub', { htmlText: htmlContent })
    } catch (err) {
      console.error('Failed to inject Furigana:', err)
      return htmlContent
    }
  }
}
