import {
  createEnvironmentInjector,
  EnvironmentInjector,
  runInInjectionContext,
} from '@angular/core'
import { Router } from '@angular/router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { NotebookService } from '../notebook/services/notebook.service'
import { PomodoroControlService } from '../pomodoro/services/pomodoro-control.service'
import { CommandPaletteService } from '../../core/services/command-palette.service'
import { CommandPaletteComponent } from './command-palette.component'

describe('CommandPaletteComponent', () => {
  let component: CommandPaletteComponent
  let router: { navigate: ReturnType<typeof vi.fn> }
  let notebook: { requestNewNote: ReturnType<typeof vi.fn> }
  let pomodoro: { requestStart: ReturnType<typeof vi.fn> }
  let palette: CommandPaletteService

  beforeEach(() => {
    router = { navigate: vi.fn().mockResolvedValue(true) }
    notebook = { requestNewNote: vi.fn() }
    pomodoro = { requestStart: vi.fn() }
    palette = new CommandPaletteService()
    const providers = [
      { provide: Router, useValue: router },
      { provide: NotebookService, useValue: notebook },
      { provide: PomodoroControlService, useValue: pomodoro },
      { provide: CommandPaletteService, useValue: palette },
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
})
