import { describe, it, expect, vi, beforeEach } from 'vitest'
import { CalendarViewComponent } from './calendar-view.component'
import { CalendarService } from '../../services/calendar.service'
import { EnvironmentInjector, createEnvironmentInjector, runInInjectionContext, signal } from '@angular/core'
import { ReviewForecastEntry, StudyHistoryEntry } from '../../models/calendar.model'
import { NotebookEntry } from '../../../notebook/models/notebook.model'

describe('CalendarViewComponent', () => {
  let component: CalendarViewComponent
  let calendarServiceMock: Partial<CalendarService>

  const mockForecast = signal<ReviewForecastEntry[]>([
    { date: '2026-10-15', count: 25 },
    { date: '2026-10-16', count: 12 },
  ])

  const mockHistory = signal<StudyHistoryEntry[]>([
    { date: '2026-10-10', count: 40 },
    { date: '2026-10-11', count: 18 },
  ])

  const mockNotebook = signal<NotebookEntry[]>([
    {
      id: 'note-1',
      title: 'Nota de Gramática',
      content_md: 'Partícula は vs が',
      created_at: '1792065600',
      updated_at: '1792065600',
    },
  ])

  beforeEach(() => {
    vi.clearAllMocks()

    calendarServiceMock = {
      forecast: mockForecast,
      studyHistory: mockHistory,
      notebookEntries: mockNotebook,
      isLoading: signal(false),
      loadMonthData: vi.fn().mockResolvedValue(undefined),
    }

    const providers = [{ provide: CalendarService, useValue: calendarServiceMock }]
    const parentInjector = {
      get: (token: unknown) => providers.find((p) => p.provide === token)?.useValue ?? null,
    } as unknown as EnvironmentInjector

    const injector = createEnvironmentInjector([], parentInjector)

    runInInjectionContext(injector, () => {
      component = new CalendarViewComponent()
    })
  })

  it('should initialize and load current month data', async () => {
    await component.ngOnInit()
    expect(calendarServiceMock.loadMonthData).toHaveBeenCalled()
    expect(component.calendarCells().length).toBeGreaterThanOrEqual(28)
  })

  it('should change month when calling nextMonth and prevMonth', async () => {
    const initialMonth = component.currentMonth()
    await component.nextMonth()
    expect(component.currentMonth()).toBe(initialMonth === 12 ? 1 : initialMonth + 1)

    await component.prevMonth()
    expect(component.currentMonth()).toBe(initialMonth)
  })

  it('should compute heatmap color correctly based on review volume', () => {
    const dummyCell = {
      date: '2026-10-10',
      dayNumber: 10,
      isCurrentMonth: true,
      isToday: false,
      isPast: true,
      isFuture: false,
      completedCount: 20,
      dueCount: 0,
      notes: [],
    }
    const color = component.getPastHeatmapColor(dummyCell)
    expect(color).toContain('rgba(16, 185, 129')
  })
})
