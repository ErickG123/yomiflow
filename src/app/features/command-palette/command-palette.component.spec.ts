import {
  createEnvironmentInjector,
  EnvironmentInjector,
  runInInjectionContext,
  signal,
} from '@angular/core'
import { Router } from '@angular/router'
import { Subject } from 'rxjs'
import { TranslocoService } from '@ngneat/transloco'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { NotebookService } from '../notebook/services/notebook.service'
import { PomodoroControlService } from '../pomodoro/services/pomodoro-control.service'
import { CommandPaletteService } from '../../core/services/command-palette.service'
import { GlobalSearchService } from '../../core/services/global-search.service'
import { SearchResult } from '../../core/models/search-result.model'
import { CommandPaletteComponent } from './command-palette.component'
import { LayoutStateService } from '../../layouts/services/layout-state.service'

describe('CommandPaletteComponent', () => {
  let component: CommandPaletteComponent
  let router: { navigate: ReturnType<typeof vi.fn> }
  let notebook: { requestNewNote: ReturnType<typeof vi.fn> }
  let pomodoro: { requestStart: ReturnType<typeof vi.fn> }
  let palette: CommandPaletteService
  let activeLang: string
  let langChanges: Subject<string>
  let layoutState: LayoutStateService
  let globalSearch: {
    results: ReturnType<typeof signal<SearchResult[]>>
    isSearching: ReturnType<typeof signal<boolean>>
    clear: ReturnType<typeof vi.fn>
    search: ReturnType<typeof vi.fn>
    snippetParts: ReturnType<typeof vi.fn>
  }

  beforeEach(() => {
    router = { navigate: vi.fn().mockResolvedValue(true) }
    notebook = { requestNewNote: vi.fn() }
    pomodoro = { requestStart: vi.fn() }
    palette = new CommandPaletteService()
    activeLang = 'en'
    langChanges = new Subject<string>()
    layoutState = new LayoutStateService()
    const dictionaries: Record<string, Record<string, string>> = {
      en: {
        'palette.commands.dashboard': 'Go to Dashboard',
        'palette.commands.newNote': 'New Note',
        'palette.commands.pomodoro': 'Start Pomodoro',
        'palette.commands.notebook': 'Go to Notebook',
        'palette.commands.study': 'Open SRS Reviews',
        'palette.commands.dictionary': 'Open Dictionary',
        'palette.commands.settings': 'Settings',
        'palette.terms.dashboard': 'dashboard|home',
        'palette.terms.newNote': 'note|notebook|create note',
        'palette.terms.pomodoro': 'focus|timer',
        'palette.terms.notebook': 'notes|notebook',
        'palette.terms.study': 'study|review|flashcards|srs',
        'palette.terms.dictionary': 'search|words',
        'palette.terms.settings': 'settings|preferences',
      },
      pt: {
        'palette.commands.settings': 'Configurações',
        'palette.terms.settings': 'configurações|preferências',
      },
      es: {
        'palette.commands.settings': 'Ajustes',
        'palette.terms.settings': 'ajustes|configuración|preferencias',
      },
    }
    const transloco = {
      langChanges$: langChanges,
      getActiveLang: () => activeLang,
      translate: (key: string) => dictionaries[activeLang][key] ?? key,
    }
    globalSearch = {
      results: signal<SearchResult[]>([]),
      isSearching: signal(false),
      clear: vi.fn(),
      search: vi.fn(),
      snippetParts: vi.fn((snippet: string) => [{ text: snippet, highlighted: false }]),
    }
    const providers = [
      { provide: Router, useValue: router },
      { provide: TranslocoService, useValue: transloco },
      { provide: NotebookService, useValue: notebook },
      { provide: PomodoroControlService, useValue: pomodoro },
      { provide: CommandPaletteService, useValue: palette },
      { provide: GlobalSearchService, useValue: globalSearch },
      { provide: LayoutStateService, useValue: layoutState },
    ]
    const parentInjector = {
      get: (token: unknown) =>
        providers.find((provider) => provider.provide === token)?.useValue ?? null,
    } as unknown as EnvironmentInjector
    const injector = createEnvironmentInjector([], parentInjector)
    runInInjectionContext(injector, () => {
      component = new CommandPaletteComponent()
    })
  })

  afterEach(() => component.ngOnDestroy())

  it('filters translated command names and terms in the active language', () => {
    component.onQueryChange({ target: { value: 'pomodoro' } } as unknown as Event)
    expect(component.filteredCommands().map((command) => command.id)).toEqual(['pomodoro'])

    component.onQueryChange({ target: { value: 'create note' } } as unknown as Event)
    expect(component.filteredCommands().map((command) => command.id)).toEqual(['new-note'])

    component.onQueryChange({ target: { value: 'settings' } } as unknown as Event)
    expect(component.filteredCommands().map((command) => command.id)).toEqual(['settings'])
    activeLang = 'pt'
    langChanges.next('pt')
    expect(component.filteredCommands()).toEqual([])
    component.onQueryChange({ target: { value: 'configurações' } } as unknown as Event)
    expect(component.filteredCommands().map((command) => command.id)).toEqual(['settings'])
    activeLang = 'es'
    langChanges.next('es')
    expect(component.filteredCommands()).toEqual([])
    component.onQueryChange({ target: { value: 'ajustes' } } as unknown as Event)
    expect(component.filteredCommands().map((command) => command.id)).toEqual(['settings'])
  })

  it('executes the selected command with Enter and closes the palette', () => {
    palette.open()
    component.onQueryChange({ target: { value: 'dashboard' } } as unknown as Event)
    component.handleKeydown(new KeyboardEvent('keydown', { key: 'Enter' }))

    expect(router.navigate).toHaveBeenCalledWith(['/dashboard'])
    expect(palette.isOpen()).toBe(false)
  })

  it('requests a note and starts Pomodoro from their commands', () => {
    component.execute(component.commands.find((command) => command.id === 'new-note')!)
    component.execute(component.commands.find((command) => command.id === 'pomodoro')!)

    expect(notebook.requestNewNote).toHaveBeenCalledOnce()
    expect(router.navigate).toHaveBeenCalledWith(['/notebook'])
    expect(pomodoro.requestStart).toHaveBeenCalledOnce()
  })

  it('opens global settings from its palette command', () => {
    component.execute(component.commands.find((command) => command.id === 'settings')!)
    expect(layoutState.isSettingsOpen()).toBe(true)
  })

  it('closes and clears the query on Escape', () => {
    palette.open()
    component.onQueryChange({ target: { value: 'study' } } as unknown as Event)
    component.handleKeydown(new KeyboardEvent('keydown', { key: 'Escape' }))

    expect(palette.isOpen()).toBe(false)
    expect(component.query()).toBe('')
  })

  it('navigates search results to their corresponding feature', () => {
    const note = {
      id: 'note:entry-8',
      sourceType: 'note' as const,
      title: 'Grammar note',
      snippet: 'Example',
    }

    component.openSearchResult(note)

    expect(router.navigate).toHaveBeenCalledWith(['/notebook'], {
      queryParams: { noteId: 'entry-8' },
    })
    expect(palette.isOpen()).toBe(false)
  })
})
