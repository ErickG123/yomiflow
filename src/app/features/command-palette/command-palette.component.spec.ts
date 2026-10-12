import {
  createEnvironmentInjector,
  EnvironmentInjector,
  runInInjectionContext,
  signal,
} from '@angular/core'
import { Router } from '@angular/router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { NotebookService } from '../notebook/services/notebook.service'
import { PomodoroControlService } from '../pomodoro/services/pomodoro-control.service'
import { CommandPaletteService } from '../../core/services/command-palette.service'
import { GlobalSearchService } from '../../core/services/global-search.service'
import { SearchResult } from '../../core/models/search-result.model'
import { CommandPaletteComponent } from './command-palette.component'

describe('CommandPaletteComponent', () => {
  let component: CommandPaletteComponent
  let router: { navigate: ReturnType<typeof vi.fn> }
  let notebook: { requestNewNote: ReturnType<typeof vi.fn> }
  let pomodoro: { requestStart: ReturnType<typeof vi.fn> }
  let palette: CommandPaletteService
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
    globalSearch = {
      results: signal<SearchResult[]>([]),
      isSearching: signal(false),
      clear: vi.fn(),
      search: vi.fn(),
      snippetParts: vi.fn((snippet: string) => [{ text: snippet, highlighted: false }]),
    }
    const providers = [
      { provide: Router, useValue: router },
      { provide: NotebookService, useValue: notebook },
      { provide: PomodoroControlService, useValue: pomodoro },
      { provide: CommandPaletteService, useValue: palette },
      { provide: GlobalSearchService, useValue: globalSearch },
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

  it('filters commands reactively by label and keywords', () => {
    component.onQueryChange({ target: { value: 'pomodoro' } } as unknown as Event)
    expect(component.filteredCommands().map((command) => command.id)).toEqual(['pomodoro'])

    component.onQueryChange({ target: { value: 'anotação' } } as unknown as Event)
    expect(component.filteredCommands().map((command) => command.id)).toEqual(['new-note'])
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
