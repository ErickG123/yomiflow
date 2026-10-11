import { describe, it, expect, vi, beforeEach } from 'vitest'
import { JlptAnalyticsComponent } from './jlpt-analytics.component'
import { JlptStatsService } from '../../services/jlpt-stats.service'
import { signal, EnvironmentInjector, createEnvironmentInjector, runInInjectionContext } from '@angular/core'
import { Badge, JlptAnalyticsStats } from '../../models/jlpt-stats.model'

describe('JlptAnalyticsComponent', () => {
  let component: JlptAnalyticsComponent
  let statsServiceMock: {
    stats: ReturnType<typeof signal<JlptAnalyticsStats | null>>
    badges: ReturnType<typeof signal<Badge[]>>
    isLoading: ReturnType<typeof signal<boolean>>
    error: ReturnType<typeof signal<string | null>>
    fetchJlptStats: ReturnType<typeof vi.fn>
    fetchBadges: ReturnType<typeof vi.fn>
  }

  beforeEach(() => {
    vi.restoreAllMocks()

    statsServiceMock = {
      stats: signal({
        levels: [
          {
            level: 5,
            level_name: 'N5',
            total_kanji: 103,
            learned_kanji: 20,
            kanji_percentage: 19.4,
            total_vocab: 800,
            learned_vocab: 150,
            vocab_percentage: 18.8,
            overall_percentage: 19.1,
          },
        ],
        total_kanji_learned: 20,
        total_vocab_learned: 150,
      }),
      badges: signal([]),
      isLoading: signal(false),
      error: signal(null),
      fetchJlptStats: vi.fn().mockResolvedValue(undefined),
      fetchBadges: vi.fn().mockResolvedValue(undefined),
    }

    const providers = [{ provide: JlptStatsService, useValue: statsServiceMock }]
    const parentInjector = {
      get: (token: unknown) => providers.find((p) => p.provide === token)?.useValue ?? null,
    } as unknown as EnvironmentInjector
    const injector = createEnvironmentInjector([], parentInjector)

    runInInjectionContext(injector, () => {
      component = new JlptAnalyticsComponent()
    })
  })

  it('should initialize and call fetchJlptStats', async () => {
    expect(component).toBeTruthy()
    await component.ngOnInit()
    expect(statsServiceMock.fetchJlptStats).toHaveBeenCalledTimes(1)
  })

  it('should cleanly destroy chart on ngOnDestroy', () => {
    expect(() => component.ngOnDestroy()).not.toThrow()
  })
})
