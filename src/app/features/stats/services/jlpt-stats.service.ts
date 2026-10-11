import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { Badge, JlptAnalyticsStats } from '../models/jlpt-stats.model'

@Injectable({
  providedIn: 'root',
})
export class JlptStatsService {
  readonly stats = signal<JlptAnalyticsStats | null>(null)
  readonly badges = signal<Badge[]>([])
  readonly isLoading = signal<boolean>(false)
  readonly error = signal<string | null>(null)

  async fetchJlptStats(): Promise<void> {
    this.isLoading.set(true)
    this.error.set(null)
    try {
      const data = await invoke<JlptAnalyticsStats>('get_jlpt_coverage_stats')
      this.stats.set(data)
    } catch (err) {
      console.error('Failed to load JLPT stats:', err)
      this.error.set(String(err))
    } finally {
      this.isLoading.set(false)
    }
  }

  async fetchBadges(): Promise<void> {
    try {
      const badgesData = await invoke<Badge[]>('get_badges')
      this.badges.set(badgesData)
    } catch (err) {
      console.error('Failed to load achievement badges:', err)
    }
  }
}
