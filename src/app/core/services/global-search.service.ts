import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { SearchResult, SearchSnippetPart } from '../models/search-result.model'

@Injectable({ providedIn: 'root' })
export class GlobalSearchService {
  readonly results = signal<SearchResult[]>([])
  readonly isSearching = signal(false)
  private requestId = 0

  async search(query: string): Promise<void> {
    const requestId = ++this.requestId
    const trimmed = query.trim()
    if (!trimmed) {
      this.results.set([])
      this.isSearching.set(false)
      return
    }

    this.isSearching.set(true)
    try {
      const results = await invoke<SearchResult[]>('global_search', { query: trimmed })
      if (requestId === this.requestId) this.results.set(results)
    } catch (error) {
      if (requestId === this.requestId) {
        console.warn('Global search failed:', error)
        this.results.set([])
      }
    } finally {
      if (requestId === this.requestId) this.isSearching.set(false)
    }
  }

  clear(): void {
    this.requestId++
    this.results.set([])
    this.isSearching.set(false)
  }

  snippetParts(snippet: string): SearchSnippetPart[] {
    const parts: SearchSnippetPart[] = []
    let highlighted = false
    for (const segment of snippet.split(/(<mark>|<\/mark>)/g)) {
      if (segment === '<mark>') highlighted = true
      else if (segment === '</mark>') highlighted = false
      else if (segment) parts.push({ text: segment, highlighted })
    }
    return parts
  }
}
