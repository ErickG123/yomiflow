import { describe, it, expect, vi, beforeEach } from 'vitest'
import { DeckListComponent } from './deck-list.component'
import { DeckService } from '../../services/deck.service'
import { Router } from '@angular/router'
import { EnvironmentInjector, createEnvironmentInjector, runInInjectionContext, signal } from '@angular/core'
import { DeckWithCounts } from '../../models/deck.model'

describe('DeckListComponent', () => {
  let component: DeckListComponent
  let deckServiceMock: Partial<DeckService>
  let routerMock: Partial<Router>
  const mockDecksSignal = signal<DeckWithCounts[]>([
    {
      id: 'default',
      name: 'Padrão',
      description: 'Baralho principal',
      new_count: 5,
      due_count: 12,
      total_count: 30,
    },
    {
      id: 'deck-n3',
      name: 'JLPT N3',
      description: 'Vocabulário intermediário',
      new_count: 10,
      due_count: 0,
      total_count: 50,
    },
  ])

  beforeEach(() => {
    vi.clearAllMocks()

    deckServiceMock = {
      decks: mockDecksSignal,
      isLoading: signal(false),
      loadDecks: vi.fn().mockResolvedValue(mockDecksSignal()),
      createDeck: vi.fn().mockResolvedValue({ id: 'new-id', name: 'Novo Deck' }),
      deleteDeck: vi.fn().mockResolvedValue(undefined),
    }

    routerMock = {
      navigate: vi.fn().mockResolvedValue(true),
    }

    const providers = [
      { provide: DeckService, useValue: deckServiceMock },
      { provide: Router, useValue: routerMock },
    ]

    const parentInjector = {
      get: (token: unknown) => providers.find((p) => p.provide === token)?.useValue ?? null,
    } as unknown as EnvironmentInjector

    const injector = createEnvironmentInjector([], parentInjector)

    runInInjectionContext(injector, () => {
      component = new DeckListComponent()
    })
  })

  it('should initialize and load decks', async () => {
    await component.ngOnInit()
    expect(deckServiceMock.loadDecks).toHaveBeenCalled()
    expect(component.deckService.decks().length).toBe(2)
  })

  it('should toggle create modal state', () => {
    expect(component.isCreateModalOpen()).toBe(false)
    component.toggleCreateModal()
    expect(component.isCreateModalOpen()).toBe(true)
    component.toggleCreateModal()
    expect(component.isCreateModalOpen()).toBe(false)
  })

  it('should navigate to study session with deckId query param', () => {
    component.studyDeck('deck-n3')
    expect(routerMock.navigate).toHaveBeenCalledWith(['/study'], {
      queryParams: { deckId: 'deck-n3' },
    })
  })
})
