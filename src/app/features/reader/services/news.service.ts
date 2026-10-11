import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { NhkNewsItem } from '../models/news.model'

@Injectable({
  providedIn: 'root',
})
export class NewsService {
  readonly newsList = signal<NhkNewsItem[]>([])
  readonly selectedArticle = signal<NhkNewsItem | null>(null)
  readonly isLoading = signal<boolean>(false)
  readonly errorMessage = signal<string | null>(null)

  async fetchNews(): Promise<NhkNewsItem[]> {
    this.isLoading.set(true)
    this.errorMessage.set(null)
    try {
      const items = await invoke<NhkNewsItem[]>('fetch_nhk_easy_news')
      this.newsList.set(items)
      if (items.length > 0 && !this.selectedArticle()) {
        this.selectedArticle.set(items[0])
      }
      return items
    } catch (err) {
      const msg = typeof err === 'string' ? err : 'Falha ao carregar notícias do NHK'
      this.errorMessage.set(msg)
      console.error('Error fetching NHK Easy News:', err)
      return []
    } finally {
      this.isLoading.set(false)
    }
  }

  selectArticle(article: NhkNewsItem): void {
    this.selectedArticle.set(article)
  }
}
