import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { ReviewForecastEntry, StudyHistoryEntry } from '../models/calendar.model'
import { NotebookEntry } from '../../notebook/models/notebook.model'

@Injectable({
  providedIn: 'root',
})
export class CalendarService {
  readonly forecast = signal<ReviewForecastEntry[]>([])
  readonly studyHistory = signal<StudyHistoryEntry[]>([])
  readonly notebookEntries = signal<NotebookEntry[]>([])
  readonly isLoading = signal<boolean>(false)

  async loadReviewForecast(startDate: string, endDate: string): Promise<ReviewForecastEntry[]> {
    try {
      const data = await invoke<ReviewForecastEntry[]>('get_review_forecast', {
        startDate,
        endDate,
      })
      this.forecast.set(data)
      return data
    } catch (error) {
      console.error('Failed to load review forecast:', error)
      this.forecast.set([])
      return []
    }
  }

  async loadStudyHistory(year: number, month: number): Promise<StudyHistoryEntry[]> {
    try {
      const data = await invoke<StudyHistoryEntry[]>('get_study_history', {
        year,
        month,
      })
      this.studyHistory.set(data)
      return data
    } catch (error) {
      console.error('Failed to load study history:', error)
      this.studyHistory.set([])
      return []
    }
  }

  async loadNotebookEntries(): Promise<NotebookEntry[]> {
    try {
      const data = await invoke<NotebookEntry[]>('get_notebook_entries')
      this.notebookEntries.set(data)
      return data
    } catch (error) {
      console.error('Failed to load notebook entries for calendar:', error)
      this.notebookEntries.set([])
      return []
    }
  }

  async loadMonthData(year: number, month: number): Promise<void> {
    this.isLoading.set(true)
    try {
      // Calculate start and end date string bounds for forecast (cover full month + buffer)
      const startDate = `${year}-${String(month).padStart(2, '0')}-01`
      const lastDay = new Date(year, month, 0).getDate()
      const endDate = `${year}-${String(month).padStart(2, '0')}-${String(lastDay).padStart(2, '0')}`

      await Promise.all([
        this.loadReviewForecast(startDate, endDate),
        this.loadStudyHistory(year, month),
        this.loadNotebookEntries(),
      ])
    } finally {
      this.isLoading.set(false)
    }
  }
}
