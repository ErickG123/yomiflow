import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { DashboardStats } from '../models/dashboard-stats.model'

@Injectable({
  providedIn: 'root',
})
export class DashboardService {
  readonly stats = signal<DashboardStats>({
    due_today: 0,
    new_cards: 0,
    total_cards: 0,
    total_dictionary_words: 0,
    total_kanjis: 0,
    total_tags: 0,
  })

  readonly isLoading = signal<boolean>(false)

  async getStats(): Promise<DashboardStats> {
    this.isLoading.set(true)
    try {
      const data = await invoke<DashboardStats>('get_dashboard_stats')
      this.stats.set(data)
      return data
    } catch (error) {
      console.error('Failed to get dashboard stats:', error)
      return this.stats()
    } finally {
      this.isLoading.set(false)
    }
  }
}
