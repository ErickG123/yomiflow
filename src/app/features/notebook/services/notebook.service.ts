import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { Subject } from 'rxjs'
import {
  BunsetsuChunk,
  LintSuggestion,
  MorphologicalToken,
  NotebookEntry,
} from '../models/notebook.model'

@Injectable({
  providedIn: 'root',
})
export class NotebookService {
  readonly entries = signal<NotebookEntry[]>([])
  readonly currentEntry = signal<NotebookEntry | null>(null)
  readonly isAnalyzing = signal<boolean>(false)
  readonly newNoteRequested = signal(false)
  readonly newNoteRequests = new Subject<void>()

  requestNewNote(): void {
    this.newNoteRequested.set(true)
    this.newNoteRequests.next()
  }

  consumeNewNoteRequest(): void {
    this.newNoteRequested.set(false)
  }

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

  async generateSyntaxTree(text: string): Promise<BunsetsuChunk[]> {
    if (!text || !text.trim()) {
      return []
    }
    try {
      return await invoke<BunsetsuChunk[]>('generate_syntax_tree', { text })
    } catch (err) {
      console.error('Erro ao gerar árvore sintática:', err)
      return []
    }
  }

  async lintJapaneseText(text: string): Promise<LintSuggestion[]> {
    if (!text.trim()) return []
    try {
      return await invoke<LintSuggestion[]>('lint_japanese_text', { text })
    } catch (err) {
      console.warn('Falha ao verificar gramática japonesa:', err)
      return []
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

  upsertEntry(entry: NotebookEntry): void {
    this.entries.update((entries) => [
      entry,
      ...entries.filter((current) => current.id !== entry.id),
    ])
  }

  async saveEntry(
    id: string,
    title: string,
    contentHtml: string,
    deckId?: string | null,
    nextReviewDate?: number | null,
    srsStage?: number | null,
  ): Promise<void> {
    try {
      await invoke('save_notebook_entry', {
        id,
        title,
        contentMd: contentHtml,
        deckId: deckId ?? null,
        nextReviewDate: nextReviewDate ?? null,
        srsStage: srsStage ?? 0,
      })
    } catch (err) {
      console.error('Erro ao guardar nota do caderno:', err)
      throw err
    }
  }

  async deleteEntry(id: string): Promise<void> {
    try {
      await invoke('delete_notebook_entry', { id })
      this.entries.update((entries) => entries.filter((entry) => entry.id !== id))
      if (this.currentEntry()?.id === id) this.currentEntry.set(null)
    } catch (err) {
      console.error('Erro ao excluir nota do caderno:', err)
      throw err
    }
  }

  async linkToDeck(id: string, deckId: string | null): Promise<void> {
    try {
      await invoke('link_notebook_to_deck', {
        id,
        deckId,
      })
      await this.loadEntries()
    } catch (err) {
      console.error('Erro ao vincular nota ao baralho:', err)
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
