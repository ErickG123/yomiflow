import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { DashboardStats, HeatmapEntry, StreakStats } from '../models/dashboard-stats.model'

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

  readonly heatmapData = signal<HeatmapEntry[]>([])
  readonly streakStats = signal<StreakStats>({
    current_streak: 0,
    max_streak: 0,
  })

  readonly isLoading = signal<boolean>(false)

  async getStats(deckId?: string | null): Promise<DashboardStats> {
    this.isLoading.set(true)
    try {
      const data = await invoke<DashboardStats>('get_dashboard_stats', { deckId: deckId ?? null })
      this.stats.set(data)
      return data
    } catch (error) {
      console.error('Failed to get dashboard stats:', error)
      return this.stats()
    } finally {
      this.isLoading.set(false)
    }
  }

  async getHeatmap(): Promise<HeatmapEntry[]> {
    try {
      const data = await invoke<HeatmapEntry[]>('get_study_heatmap')
      this.heatmapData.set(data)
      return data
    } catch (error) {
      console.error('Failed to get study heatmap:', error)
      return []
    }
  }

  async getStreaks(): Promise<StreakStats> {
    try {
      const data = await invoke<StreakStats>('get_study_streaks')
      this.streakStats.set(data)
      return data
    } catch (error) {
      console.error('Failed to get study streaks:', error)
      return { current_streak: 0, max_streak: 0 }
    }
  }
}
