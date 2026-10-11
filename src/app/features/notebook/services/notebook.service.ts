import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { MorphologicalToken, NotebookEntry } from '../models/notebook.model'

@Injectable({
  providedIn: 'root',
})
export class NotebookService {
  readonly entries = signal<NotebookEntry[]>([])
  readonly currentEntry = signal<NotebookEntry | null>(null)
  readonly isAnalyzing = signal<boolean>(false)

  async analyzeText(text: string): Promise<MorphologicalToken[]> {
    if (!text || !text.trim()) {
      return []
    }
    this.isAnalyzing.set(true)
    try {
      return await invoke<MorphologicalToken[]>('analyze_japanese_text', { text })
    } catch (err) {
      console.error('Erro na análise morfológica Lindera:', err)
      return []
    } finally {
      this.isAnalyzing.set(false)
    }
  }

  async loadEntries(): Promise<NotebookEntry[]> {
    try {
      const data = await invoke<NotebookEntry[]>('get_notebook_entries')
      this.entries.set(data)
      return data
    } catch (err) {
      console.warn('Erro ao carregar notas do caderno:', err)
      return []
    }
  }

  async saveEntry(id: string, title: string, contentMd: string): Promise<void> {
    try {
      await invoke('save_notebook_entry', {
        id,
        title,
        contentMd,
      })
      await this.loadEntries()
    } catch (err) {
      console.error('Erro ao guardar nota do caderno:', err)
      throw err
    }
  }

  formatTokensToHtml(tokens: MorphologicalToken[]): string {
    return tokens
      .map((t) => {
        let colorClass: string
        if (t.pos === 'verb') {
          colorClass = 'text-red-400 font-medium'
        } else if (t.pos === 'particle') {
          colorClass = 'text-blue-400 font-medium'
        } else if (t.pos === 'noun') {
          colorClass = 'text-green-400 font-medium'
        } else {
          colorClass = 'text-zinc-300'
        }

        const readingAttr = t.reading ? ` title="${t.reading} (${t.pos})"` : ` title="${t.pos}"`
        return `<span class="${colorClass}"${readingAttr}>${t.surface}</span>`
      })
      .join('')
  }
}
