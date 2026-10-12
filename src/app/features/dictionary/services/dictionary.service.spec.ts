import { describe, it, expect, vi, beforeEach } from 'vitest'
import { DictionaryService } from './dictionary.service'
import { AppStateService } from '../../../core/services/app-state.service'
import { DeckService } from '../../decks/services/deck.service'
import { signal } from '@angular/core'
import {
  EnvironmentInjector,
  createEnvironmentInjector,
  runInInjectionContext,
} from '@angular/core'
import * as tauriCore from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('DictionaryService', () => {
  let service: DictionaryService
  let appStateMock: AppStateService
  let deckServiceMock: { targetDeckId: ReturnType<typeof signal<string>> }

  beforeEach(() => {
    vi.clearAllMocks()
    appStateMock = {
      isProcessing: { set: vi.fn() },
      progressMessage: { set: vi.fn() },
    } as unknown as AppStateService
    deckServiceMock = { targetDeckId: signal('default') }

    const providers = [
      { provide: AppStateService, useValue: appStateMock },
      { provide: DeckService, useValue: deckServiceMock },
    ]
    const parentInjector = {
      get: (token: unknown) => providers.find((p) => p.provide === token)?.useValue ?? null,
    } as unknown as EnvironmentInjector
    const injector = createEnvironmentInjector([], parentInjector)

    runInInjectionContext(injector, () => {
      service = new DictionaryService()
    })
  })

  it('should initialize with empty results and idle state', () => {
    expect(service.searchResults()).toEqual([])
    expect(service.kanjiSearchResults()).toEqual([])
    expect(service.isLoading()).toBe(false)
  })

  it('should clear results immediately when query is blank without invoking backend', async () => {
    const res = await service.search('   ')

    expect(res).toEqual({ dictResults: [], kanjiResults: [] })
    expect(service.searchResults()).toEqual([])
    expect(service.kanjiSearchResults()).toEqual([])
    expect(service.isLoading()).toBe(false)
    expect(tauriCore.invoke).not.toHaveBeenCalled()
  })

  it('should update searchResults and kanjiSearchResults when backend returns entries', async () => {
    const mockDict = [{ id: 1, kanji: '猫', reading: 'ねこ', meaning: 'cat', in_srs: false }]
    const mockKanji = [
      { character: '猫', onyomi: 'ビョウ', kunyomi: 'ねこ', meaning: 'cat', jlpt_level: 3 },
    ]

    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'search_dictionary') return mockDict
      if (cmd === 'search_kanjis') return mockKanji
      return null
    })

    const result = await service.search('猫', 'deck-n3')

    expect(result.dictResults).toEqual(mockDict)
    expect(result.kanjiResults).toEqual(mockKanji)
    expect(service.searchResults()).toEqual(mockDict)
    expect(service.kanjiSearchResults()).toEqual(mockKanji)
    expect(service.isLoading()).toBe(false)
  })

  it('should handle search errors gracefully and maintain empty state', async () => {
    vi.mocked(tauriCore.invoke).mockRejectedValue(new Error('Backend error'))

    const result = await service.search('something')

    expect(result.dictResults).toEqual([])
    expect(result.kanjiResults).toEqual([])
    expect(service.searchResults()).toEqual([])
    expect(service.isLoading()).toBe(false)
  })

  it('allows adding the same dictionary entry to more than one deck', async () => {
    vi.mocked(tauriCore.invoke).mockResolvedValue(undefined)

    await service.addToSrs(42, 'default')
    await service.addToSrs(42, 'deck-n3')

    expect(tauriCore.invoke).toHaveBeenNthCalledWith(1, 'add_to_srs', {
      entryId: 42,
      deckId: 'default',
    })
    expect(tauriCore.invoke).toHaveBeenNthCalledWith(2, 'add_to_srs', {
      entryId: 42,
      deckId: 'deck-n3',
    })
    expect(service.addedCardIds()).toContain(42)
  })

  it('uses the target deck when no deck is explicitly selected', async () => {
    vi.mocked(tauriCore.invoke).mockResolvedValue(undefined)
    deckServiceMock.targetDeckId.set('deck-n3')

    await service.addToSrs(12)

    expect(tauriCore.invoke).toHaveBeenCalledWith('add_to_srs', {
      entryId: 12,
      deckId: 'deck-n3',
    })
  })
})
