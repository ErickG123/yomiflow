import { describe, it, expect, vi, beforeEach } from 'vitest'
import { NotebookEditorComponent } from './notebook-editor.component'
import { NotebookService } from '../../services/notebook.service'
import { DeckService } from '../../../decks/services/deck.service'
import { DeckWithCounts } from '../../../decks/models/deck.model'
import {
  signal,
  EnvironmentInjector,
  createEnvironmentInjector,
  runInInjectionContext,
} from '@angular/core'
import { MorphologicalToken, NotebookEntry } from '../../models/notebook.model'
import { Subject } from 'rxjs'

describe('NotebookEditorComponent', () => {
  let component: NotebookEditorComponent
  let notebookServiceMock: {
    entries: ReturnType<typeof signal<NotebookEntry[]>>
    currentEntry: ReturnType<typeof signal<NotebookEntry | null>>
    isAnalyzing: ReturnType<typeof signal<boolean>>
    newNoteRequested: ReturnType<typeof signal<boolean>>
    newNoteRequests: Subject<void>
    consumeNewNoteRequest: ReturnType<typeof vi.fn>
    analyzeText: ReturnType<typeof vi.fn>
    generateSyntaxTree: ReturnType<typeof vi.fn>
    loadEntries: ReturnType<typeof vi.fn>
    saveEntry: ReturnType<typeof vi.fn>
    formatTokensToHtml: ReturnType<typeof vi.fn>
  }
  let deckServiceMock: {
    decks: ReturnType<typeof signal<DeckWithCounts[]>>
    loadDecks: ReturnType<typeof vi.fn>
  }

  const mockTokens: MorphologicalToken[] = [
    { surface: '私', pos: 'noun', reading: 'ワタシ' },
    { surface: 'は', pos: 'particle', reading: 'ハ' },
    { surface: '学生', pos: 'noun', reading: 'ガクセイ' },
    { surface: 'です', pos: 'auxiliary', reading: 'デス' },
  ]

  beforeEach(() => {
    vi.restoreAllMocks()

    notebookServiceMock = {
      entries: signal([]),
      currentEntry: signal(null),
      isAnalyzing: signal(false),
      newNoteRequested: signal(false),
      newNoteRequests: new Subject<void>(),
      consumeNewNoteRequest: vi.fn(),
      analyzeText: vi.fn().mockResolvedValue(mockTokens),
      generateSyntaxTree: vi.fn().mockResolvedValue([
        { id: 1, text: '私は', pos: 'noun+particle', target_id: 3 },
        { id: 2, text: '学生', pos: 'noun', target_id: 3 },
        { id: 3, text: 'です', pos: 'auxiliary', target_id: null },
      ]),
      loadEntries: vi.fn().mockResolvedValue([]),
      saveEntry: vi.fn().mockResolvedValue(undefined),
      formatTokensToHtml: vi
        .fn()
        .mockReturnValue(
          '<span class="text-green-400 font-medium">私</span><span class="text-blue-400 font-medium">は</span><span class="text-green-400 font-medium">学生</span><span class="text-zinc-300">です</span>',
        ),
    }

    deckServiceMock = {
      decks: signal([]),
      loadDecks: vi.fn().mockResolvedValue([]),
    }

    const applicationRefMock = {
      attachView: vi.fn(),
      detachView: vi.fn(),
    }

    const providers = [
      { provide: NotebookService, useValue: notebookServiceMock },
      { provide: DeckService, useValue: deckServiceMock },
      { provide: 'ApplicationRef', useValue: applicationRefMock },
    ]
    const parentInjector = {
      get: (token: unknown) => {
        if (typeof token === 'function' && token.name === 'ApplicationRef') {
          return applicationRefMock
        }
        return providers.find((p) => p.provide === token)?.useValue ?? null
      },
    } as unknown as EnvironmentInjector
    const injector = createEnvironmentInjector([], parentInjector)

    runInInjectionContext(injector, () => {
      component = new NotebookEditorComponent()
    })
  })

  it('should initialize with default sample Japanese markdown content and load entries', async () => {
    await component.ngOnInit()

    expect(notebookServiceMock.loadEntries).toHaveBeenCalledTimes(1)
    expect(deckServiceMock.loadDecks).toHaveBeenCalledTimes(1)
    expect(component.markdownContent()).toContain('私は学生です')
    expect(component.tokens().length).toBe(4)
  })

  it('should extract markdown content and save notebook entry via service', async () => {
    component.title.set('Capítulo 1: Introdução')
    component.markdownContent.set('# Lição 1\nこれはペンです。')

    await component.saveNote()

    expect(notebookServiceMock.saveEntry).toHaveBeenCalledWith(
      component.currentId(),
      'Capítulo 1: Introdução',
      '# Lição 1\nこれはペンです。',
      null,
      null,
      0,
    )
  })

  it('should reset editor and state when creating a new note', () => {
    component.createNewNote()

    expect(component.title()).toBe('Nova Anotação')
    expect(component.markdownContent()).toBe('')
    expect(component.tokens().length).toBe(0)
  })

  it('should format tokens to colored HTML based on morphological categories', () => {
    const formatted = component.formattedHtml()
    expect(formatted).toContain('text-green-400')
    expect(formatted).toContain('text-blue-400')
  })

  it('should toggle analysis mode between auto and manual', () => {
    expect(component.analysisMode()).toBe('manual')
    component.setAnalysisMode('auto')
    expect(component.analysisMode()).toBe('auto')
    component.setAnalysisMode('manual')
    expect(component.analysisMode()).toBe('manual')
  })

  it('should trigger morphological analysis on manual button click', async () => {
    notebookServiceMock.analyzeText.mockClear()
    component.markdownContent.set('日本語')

    await component.analyzeManually()

    expect(notebookServiceMock.analyzeText).toHaveBeenCalledWith('日本語')
    expect(notebookServiceMock.generateSyntaxTree).toHaveBeenCalledWith('日本語')
  })

  it('should toggle syntax tree visibility and update syntaxTreeData', () => {
    expect(component.showSyntaxTree()).toBe(false)
    component.toggleSyntaxTree()
    expect(component.showSyntaxTree()).toBe(true)
    component.toggleSyntaxTree()
    expect(component.showSyntaxTree()).toBe(false)
  })
})
