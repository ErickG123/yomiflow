import { NotebookEntry } from '../../notebook/models/notebook.model'

export interface ReviewForecastEntry {
  date: string
  count: number
}

export interface StudyHistoryEntry {
  date: string
  count: number
}

export interface CalendarDayCell {
  date: string // "YYYY-MM-DD"
  dayNumber: number
  isCurrentMonth: boolean
  isToday: boolean
  isPast: boolean
  isFuture: boolean
  completedCount: number // reviews finished on this day (past/today)
  dueCount: number // cards scheduled for this day (future/today)
  notes: NotebookEntry[]
}

export interface DayDetailSummary {
  date: string
  completedCount: number
  scheduledCount: number
  notes: NotebookEntry[]
}
