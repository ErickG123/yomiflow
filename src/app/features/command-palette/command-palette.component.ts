import { Component, computed, inject, signal } from '@angular/core'
import { Router } from '@angular/router'
import { CommandPaletteCommand } from '../../core/models/command-palette.model'
import { CommandPaletteService } from '../../core/services/command-palette.service'
import { NotebookService } from '../notebook/services/notebook.service'
import { PomodoroControlService } from '../pomodoro/services/pomodoro-control.service'

@Component({
  selector: 'app-command-palette',
  standalone: true,
  templateUrl: './command-palette.component.html',
})
export class CommandPaletteComponent {
  private readonly router = inject(Router)
  private readonly notebook = inject(NotebookService)
  private readonly pomodoro = inject(PomodoroControlService)
  readonly palette = inject(CommandPaletteService)
  readonly query = signal('')
  readonly selectedIndex = signal(0)

  readonly commands: CommandPaletteCommand[] = [
    {
      id: 'dashboard',
      label: 'Ir para Dashboard',
      keywords: ['início', 'home'],
      action: 'dashboard',
    },
    {
      id: 'new-note',
      label: 'Nova Nota',
      keywords: ['anotação', 'caderno', 'nota'],
      action: 'new-note',
    },
    { id: 'pomodoro', label: 'Iniciar Pomodoro', keywords: ['foco', 'timer'], action: 'pomodoro' },
    {
      id: 'notebook',
      label: 'Ir para Caderno',
      keywords: ['notas', 'notebook'],
      action: 'notebook',
    },
    {
      id: 'study',
      label: 'Abrir Revisões SRS',
      keywords: ['estudo', 'revisão', 'flashcards'],
      action: 'study',
    },
    {
      id: 'dictionary',
      label: 'Abrir Dicionário',
      keywords: ['buscar', 'palavras'],
      action: 'dictionary',
    },
  ]

  readonly filteredCommands = computed(() => {
    const query = this.query().trim().toLocaleLowerCase()
    if (!query) return this.commands
    return this.commands.filter((command) =>
      [command.label, ...command.keywords].some((value) =>
        value.toLocaleLowerCase().includes(query),
      ),
    )
  })

  onQueryChange(event: Event): void {
    this.query.set((event.target as HTMLInputElement).value)
    this.selectedIndex.set(0)
  }

  handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault()
      this.close()
      return
    }

    const count = this.filteredCommands().length
    if (event.key === 'ArrowDown' && count > 0) {
      event.preventDefault()
      this.selectedIndex.update((index) => (index + 1) % count)
    } else if (event.key === 'ArrowUp' && count > 0) {
      event.preventDefault()
      this.selectedIndex.update((index) => (index - 1 + count) % count)
    } else if (event.key === 'Enter') {
      event.preventDefault()
      this.executeSelected()
    }
  }

  executeSelected(): void {
    const command = this.filteredCommands()[this.selectedIndex()]
    if (command) this.execute(command)
  }

  execute(command: CommandPaletteCommand): void {
    this.close()
    switch (command.action) {
      case 'dashboard':
        void this.router.navigate(['/dashboard'])
        break
      case 'new-note':
        this.notebook.requestNewNote()
        void this.router.navigate(['/notebook'])
        break
      case 'pomodoro':
        this.pomodoro.requestStart()
        break
      case 'notebook':
        void this.router.navigate(['/notebook'])
        break
      case 'study':
        void this.router.navigate(['/study'])
        break
      case 'dictionary':
        void this.router.navigate(['/dictionary'])
        break
    }
  }

  close(): void {
    this.query.set('')
    this.selectedIndex.set(0)
    this.palette.close()
  }
}
