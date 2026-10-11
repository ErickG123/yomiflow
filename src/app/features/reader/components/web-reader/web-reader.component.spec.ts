import { describe, it, expect, vi, beforeEach } from 'vitest'
import { WebReaderComponent } from './web-reader.component'
import { NewsService } from '../../services/news.service'
import { DictionaryService } from '../../../dictionary/services/dictionary.service'
import { NotebookService } from '../../../notebook/services/notebook.service'
import { DeckService } from '../../../decks/services/deck.service'
import { signal, EnvironmentInjector, createEnvironmentInjector, runInInjectionContext } from '@angular/core'
import { NhkNewsItem } from '../../models/news.model'

describe('WebReaderComponent', () => {
  let component: WebReaderComponent

  const mockNewsItem: NhkNewsItem = {
    id: 'news-1',
    title: '日本の桜が咲きました',
    title_with_ruby: '<ruby>日本<rt>にほん</rt></ruby>の<ruby>桜<rt>さくら</rt></ruby>が<ruby>咲<rt>さ</rt></ruby>きました',
    body: '日本の桜がとてもきれいに咲きました。\nたくさんの人が花見をします。',
    body_html: '<p>日本の桜がとてもきれいに咲きました。</p><p>たくさんの人が花見をします。</p>',
    image_url: 'https://example.com/sakura.jpg',
    news_web_url: 'https://nhkeasier.com/story/1',
    published_at: '2026-10-11',
  }

  let mockNewsService: {
    newsList: ReturnType<typeof signal<NhkNewsItem[]>>
    selectedArticle: ReturnType<typeof signal<NhkNewsItem | null>>
    isLoading: ReturnType<typeof signal<boolean>>
    errorMessage: ReturnType<typeof signal<string | null>>
    fetchNews: ReturnType<typeof vi.fn>
    selectArticle: ReturnType<typeof vi.fn>
  }

  let mockDictionaryService: {
    searchResults: ReturnType<typeof signal<any[]>>
    kanjiSearchResults: ReturnType<typeof signal<any[]>>
    isLoading: ReturnType<typeof signal<boolean>>
    addedCardIds: ReturnType<typeof signal<number[]>>
    searchDictionary: ReturnType<typeof vi.fn>
    addToSrs: ReturnType<typeof vi.fn>
    addCustomWordToSrs: ReturnType<typeof vi.fn>
  }

  let mockNotebookService: {
    saveEntry: ReturnType<typeof vi.fn>
  }

  let mockDeckService: {
    decks: ReturnType<typeof signal<any[]>>
    loadDecks: ReturnType<typeof vi.fn>
  }

  beforeEach(() => {
    vi.restoreAllMocks()

    mockNewsService = {
      newsList: signal<NhkNewsItem[]>([mockNewsItem]),
      selectedArticle: signal<NhkNewsItem | null>(mockNewsItem),
      isLoading: signal<boolean>(false),
      errorMessage: signal<string | null>(null),
      fetchNews: vi.fn().mockResolvedValue([mockNewsItem]),
      selectArticle: vi.fn(),
    }

    mockDictionaryService = {
      searchResults: signal([]),
      kanjiSearchResults: signal([]),
      isLoading: signal(false),
      addedCardIds: signal([]),
      searchDictionary: vi.fn().mockResolvedValue([
        {
          id: 1,
          kanji: '桜',
          reading: 'さくら',
          meaning: 'cherry blossom',
          in_srs: false,
        },
      ]),
      addToSrs: vi.fn().mockResolvedValue(undefined),
      addCustomWordToSrs: vi.fn().mockResolvedValue(1),
    }

    mockNotebookService = {
      saveEntry: vi.fn().mockResolvedValue(undefined),
    }

    mockDeckService = {
      decks: signal([{ id: 'default', name: 'Default', description: '' }]),
      loadDecks: vi.fn().mockResolvedValue(undefined),
    }

    const providers = [
      { provide: NewsService, useValue: mockNewsService },
      { provide: DictionaryService, useValue: mockDictionaryService },
      { provide: NotebookService, useValue: mockNotebookService },
      { provide: DeckService, useValue: mockDeckService },
    ]

    const parentInjector = {
      get: (token: any) => {
        const found = providers.find((p) => p.provide === token)
        if (found) return found.useValue
        return null
      },
    } as unknown as EnvironmentInjector

    const injector = createEnvironmentInjector(providers, parentInjector)

    runInInjectionContext(injector, () => {
      component = new WebReaderComponent()
    })
  })

  it('should create WebReaderComponent', () => {
    expect(component).toBeTruthy()
  })

  it('should tokenize paragraphs into words and particles', () => {
    const tokens = component.tokenizeSentence('日本の桜')
    expect(tokens.length).toBeGreaterThan(0)
    expect(tokens.some((t) => t.isWord)).toBe(true)
  })

  it('should search word definition on word click', async () => {
    await component.onWordClick('桜')
    expect(mockDictionaryService.searchDictionary).toHaveBeenCalledWith('桜')
    expect(component.activeWord()).toBe('桜')
    expect(component.activeWordResults().length).toBe(1)
  })

  it('should add word to SRS and update state', async () => {
    const entry = {
      id: 1,
      kanji: '桜',
      reading: 'さくら',
      meaning: 'cherry blossom',
      in_srs: false,
    }
    await component.addWordToSrs(entry)
    expect(mockDictionaryService.addToSrs).toHaveBeenCalledWith(1, 'default')
    expect(component.toastMessage()).toContain('SRS')
  })

  it('should save word or article into notebook', async () => {
    await component.sendToNotebook()
    expect(mockNotebookService.saveEntry).toHaveBeenCalled()
    expect(component.toastMessage()).toContain('Caderno')
  })
})
