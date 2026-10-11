import {
  ChangeDetectionStrategy,
  Component,
  OnInit,
  computed,
  inject,
  signal,
} from '@angular/core'
import { RouterLink } from '@angular/router'
import { CalendarService } from '../../services/calendar.service'
import {
  CalendarDayCell,
  DayDetailSummary,
} from '../../models/calendar.model'

@Component({
  selector: 'app-calendar-view',
  standalone: true,
  imports: [RouterLink],
  templateUrl: './calendar-view.component.html',
  styleUrl: './calendar-view.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class CalendarViewComponent implements OnInit {
  readonly calendarService = inject(CalendarService)

  readonly currentYear = signal<number>(new Date().getFullYear())
  readonly currentMonth = signal<number>(new Date().getMonth() + 1) // 1-12
  readonly selectedDate = signal<string>(this.formatDate(new Date()))

  private readonly monthNames = [
    'Janeiro',
    'Fevereiro',
    'Março',
    'Abril',
    'Maio',
    'Junho',
    'Julho',
    'Agosto',
    'Setembro',
    'Outubro',
    'Novembro',
    'Dezembro',
  ]

  readonly currentMonthName = computed(() => {
    return this.monthNames[this.currentMonth() - 1]
  })

  // Format date to local YYYY-MM-DD
  formatDate(d: Date): string {
    const year = d.getFullYear()
    const month = String(d.getMonth() + 1).padStart(2, '0')
    const day = String(d.getDate()).padStart(2, '0')
    return `${year}-${month}-${day}`
  }

  // Parse YYYY-MM-DD string to Date object in local time
  parseDate(str: string): Date {
    const parts = str.split('-')
    return new Date(Number(parts[0]), Number(parts[1]) - 1, Number(parts[2]))
  }

  readonly calendarCells = computed<CalendarDayCell[]>(() => {
    const year = this.currentYear()
    const month = this.currentMonth()

    const todayStr = this.formatDate(new Date())

    const forecastMap = new Map<string, number>()
    for (const f of this.calendarService.forecast()) {
      forecastMap.set(f.date, f.count)
    }

    const historyMap = new Map<string, number>()
    for (const h of this.calendarService.studyHistory()) {
      historyMap.set(h.date, h.count)
    }

    const notesList = this.calendarService.notebookEntries()

    // Determine calendar grid bounds (first day of month, padding from previous month)
    const firstDay = new Date(year, month - 1, 1)
    const startingDayOfWeek = firstDay.getDay() // 0 = Sunday
    const daysInMonth = new Date(year, month, 0).getDate()

    const cells: CalendarDayCell[] = []

    // 1. Padding days from previous month
    const prevMonthLastDay = new Date(year, month - 1, 0).getDate()
    for (let i = startingDayOfWeek - 1; i >= 0; i--) {
      const prevDate = new Date(year, month - 2, prevMonthLastDay - i)
      const dateStr = this.formatDate(prevDate)
      cells.push(
        this.createCell(
          prevDate,
          dateStr,
          false,
          todayStr,
          historyMap,
          forecastMap,
          notesList
        )
      )
    }

    // 2. Days of current month
    for (let d = 1; d <= daysInMonth; d++) {
      const curDate = new Date(year, month - 1, d)
      const dateStr = this.formatDate(curDate)
      cells.push(
        this.createCell(
          curDate,
          dateStr,
          true,
          todayStr,
          historyMap,
          forecastMap,
          notesList
        )
      )
    }

    // 3. Padding days for next month to complete 35 or 42 grid cells
    const remaining = (7 - (cells.length % 7)) % 7
    for (let i = 1; i <= remaining; i++) {
      const nextDate = new Date(year, month, i)
      const dateStr = this.formatDate(nextDate)
      cells.push(
        this.createCell(
          nextDate,
          dateStr,
          false,
          todayStr,
          historyMap,
          forecastMap,
          notesList
        )
      )
    }

    return cells
  })

  private createCell(
    dateObj: Date,
    dateStr: string,
    isCurrentMonth: boolean,
    todayStr: string,
    historyMap: Map<string, number>,
    forecastMap: Map<string, number>,
    notesList: typeof this.calendarService.notebookEntries extends () => infer R ? R : never
  ): CalendarDayCell {
    const isToday = dateStr === todayStr
    const isPast = dateStr < todayStr
    const isFuture = dateStr > todayStr

    const completedCount = historyMap.get(dateStr) ?? 0
    const dueCount = isFuture || isToday ? forecastMap.get(dateStr) ?? 0 : 0

    // Match notebook entries created or updated on this date
    const matchedNotes = notesList.filter((n) => {
      // created_at can be unix seconds or ISO string
      let noteDate = ''
      if (!isNaN(Number(n.created_at))) {
        const d = new Date(Number(n.created_at) * 1000)
        noteDate = this.formatDate(d)
      } else if (n.created_at) {
        noteDate = n.created_at.substring(0, 10)
      }
      return noteDate === dateStr
    })

    return {
      date: dateStr,
      dayNumber: dateObj.getDate(),
      isCurrentMonth,
      isToday,
      isPast,
      isFuture,
      completedCount,
      dueCount,
      notes: matchedNotes,
    }
  }

  readonly selectedDaySummary = computed<DayDetailSummary>(() => {
    const sel = this.selectedDate()
    const cells = this.calendarCells()
    const found = cells.find((c) => c.date === sel)

    if (found) {
      return {
        date: found.date,
        completedCount: found.completedCount,
        scheduledCount: found.dueCount,
        notes: found.notes,
      }
    }

    return {
      date: sel,
      completedCount: 0,
      scheduledCount: 0,
      notes: [],
    }
  })

  readonly selectedDateDisplay = computed<string>(() => {
    const parts = this.selectedDate().split('-')
    if (parts.length === 3) {
      return `${parts[2]}/${parts[1]}/${parts[0]}`
    }
    return this.selectedDate()
  })

  async ngOnInit(): Promise<void> {
    await this.loadCurrentMonth()
  }

  async loadCurrentMonth(): Promise<void> {
    await this.calendarService.loadMonthData(
      this.currentYear(),
      this.currentMonth()
    )
  }

  async prevMonth(): Promise<void> {
    let m = this.currentMonth() - 1
    let y = this.currentYear()
    if (m < 1) {
      m = 12
      y -= 1
    }
    this.currentMonth.set(m)
    this.currentYear.set(y)
    await this.loadCurrentMonth()
  }

  async nextMonth(): Promise<void> {
    let m = this.currentMonth() + 1
    let y = this.currentYear()
    if (m > 12) {
      m = 1
      y += 1
    }
    this.currentMonth.set(m)
    this.currentYear.set(y)
    await this.loadCurrentMonth()
  }

  async goToToday(): Promise<void> {
    const now = new Date()
    this.currentYear.set(now.getFullYear())
    this.currentMonth.set(now.getMonth() + 1)
    this.selectedDate.set(this.formatDate(now))
    await this.loadCurrentMonth()
  }

  selectDay(cell: CalendarDayCell): void {
    this.selectedDate.set(cell.date)
  }

  getPastHeatmapColor(cell: CalendarDayCell): string {
    if (!cell.isPast && !cell.isToday) {
      return 'transparent'
    }
    const count = cell.completedCount
    if (count === 0) return 'rgba(24, 24, 27, 0.6)' // zinc-900
    if (count <= 5) return 'rgba(6, 78, 59, 0.4)' // emerald-950
    if (count <= 15) return 'rgba(4, 120, 87, 0.5)' // emerald-700
    if (count <= 30) return 'rgba(16, 185, 129, 0.6)' // emerald-500
    return 'rgba(52, 211, 153, 0.8)' // emerald-400
  }
}
