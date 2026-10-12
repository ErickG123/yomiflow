import { Component, OnDestroy, computed, inject, signal } from '@angular/core'
import { Router } from '@angular/router'
import { TranslocoPipe, TranslocoService } from '@ngneat/transloco'
import { Subscription } from 'rxjs'
import { CommandPaletteCommand } from '../../core/models/command-palette.model'
import { SearchResult } from '../../core/models/search-result.model'
import { CommandPaletteService } from '../../core/services/command-palette.service'
import { GlobalSearchService } from '../../core/services/global-search.service'
import { NotebookService } from '../notebook/services/notebook.service'
import { PomodoroControlService } from '../pomodoro/services/pomodoro-control.service'
import { LayoutStateService } from '../../layouts/services/layout-state.service'

@Component({
  selector: 'app-command-palette',
  standalone: true,
  imports: [TranslocoPipe],
  templateUrl: './command-palette.component.html',
})
export class CommandPaletteComponent implements OnDestroy {
  private readonly router = inject(Router)
  private readonly transloco = inject(TranslocoService)
  private readonly notebook = inject(NotebookService)
  private readonly pomodoro = inject(PomodoroControlService)
  private readonly layoutState = inject(LayoutStateService)
  readonly palette = inject(CommandPaletteService)
  readonly globalSearch = inject(GlobalSearchService)
  readonly query = signal('')
  readonly selectedIndex = signal(0)
  private readonly languageRevision = signal(0)
  private readonly languageSubscription: Subscription = this.transloco.langChanges$.subscribe(() =>
    this.languageRevision.update((revision) => revision + 1),
  )
  private searchTimeout: ReturnType<typeof setTimeout> | null = null

  readonly searchResults = computed(() => {
    const results = this.globalSearch.results()
    return [
      ...results.filter((result) => result.sourceType === 'card'),
      ...results.filter((result) => result.sourceType === 'note'),
      ...results.filter((result) => result.sourceType === 'news'),
    ]
  })
  readonly searchResultOffset = computed(() => this.filteredCommands().length)
  readonly cardResults = computed(() =>
    this.searchResults().filter((result) => result.sourceType === 'card'),
  )
  readonly noteResults = computed(() =>
    this.searchResults().filter((result) => result.sourceType === 'note'),
  )
  readonly newsResults = computed(() =>
    this.searchResults().filter((result) => result.sourceType === 'news'),
  )
  readonly resultGroups = computed(() =>
    [
      { sourceType: 'card', labelKey: 'palette.groups.cards', results: this.cardResults() },
      { sourceType: 'note', labelKey: 'palette.groups.notes', results: this.noteResults() },
      { sourceType: 'news', labelKey: 'palette.groups.news', results: this.newsResults() },
    ].filter((group) => group.results.length > 0),
  )

  readonly commands: CommandPaletteCommand[] = [
    {
      id: 'dashboard',
      nameKey: 'palette.commands.dashboard',
      keywordsKey: 'palette.terms.dashboard',
      action: 'dashboard',
    },
    {
      id: 'new-note',
      nameKey: 'palette.commands.newNote',
      keywordsKey: 'palette.terms.newNote',
      action: 'new-note',
    },
    {
      id: 'pomodoro',
      nameKey: 'palette.commands.pomodoro',
      keywordsKey: 'palette.terms.pomodoro',
      action: 'pomodoro',
    },
    {
      id: 'notebook',
      nameKey: 'palette.commands.notebook',
      keywordsKey: 'palette.terms.notebook',
      action: 'notebook',
    },
    {
      id: 'study',
      nameKey: 'palette.commands.study',
      keywordsKey: 'palette.terms.study',
      action: 'study',
    },
    {
      id: 'dictionary',
      nameKey: 'palette.commands.dictionary',
      keywordsKey: 'palette.terms.dictionary',
      action: 'dictionary',
    },
    {
      id: 'settings',
      nameKey: 'palette.commands.settings',
      keywordsKey: 'palette.terms.settings',
      action: 'settings',
    },
  ]

  readonly filteredCommands = computed(() => {
    this.languageRevision()
    const query = this.query().trim().toLocaleLowerCase(this.transloco.getActiveLang())
    if (!query) return this.commands
    return this.commands.filter((command) => {
      const translatedName = this.transloco.translate(command.nameKey)
      const translatedTerms = this.transloco.translate(command.keywordsKey)
      return [translatedName, ...translatedTerms.split('|')].some((value) =>
        value.trim().toLocaleLowerCase(this.transloco.getActiveLang()).includes(query),
      )
    })
  })

  onQueryChange(event: Event): void {
    const value = (event.target as HTMLInputElement).value
    this.query.set(value)
    this.selectedIndex.set(0)
    if (this.searchTimeout) clearTimeout(this.searchTimeout)
    this.globalSearch.clear()
    if (value.trim()) {
      this.searchTimeout = setTimeout(() => void this.globalSearch.search(value), 200)
    }
  }

  handleKeydown(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault()
      this.close()
      return
    }

    const count = this.filteredCommands().length + this.searchResults().length
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
    if (this.query().trim()) {
      const commands = this.filteredCommands()
      const selectedIndex = this.selectedIndex()
      if (selectedIndex < commands.length) {
        this.execute(commands[selectedIndex])
        return
      }
      const result = this.searchResults()[selectedIndex - commands.length]
      if (result) this.openSearchResult(result)
      return
    }
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
      case 'settings':
        this.layoutState.isSettingsOpen.set(true)
        break
    }
  }

  close(): void {
    if (this.searchTimeout) clearTimeout(this.searchTimeout)
    this.searchTimeout = null
    this.globalSearch.clear()
    this.query.set('')
    this.selectedIndex.set(0)
    this.palette.close()
  }

  openSearchResult(result: SearchResult): void {
    this.close()
    const id = result.id.slice(result.id.indexOf(':') + 1)
    if (result.sourceType === 'card') {
      void this.router.navigate(['/dictionary'], { queryParams: { search: result.title } })
    } else if (result.sourceType === 'note') {
      void this.router.navigate(['/notebook'], { queryParams: { noteId: id } })
    } else {
      void this.router.navigate(['/web'], { queryParams: { articleId: id } })
    }
  }

  ngOnDestroy(): void {
    this.languageSubscription.unsubscribe()
    if (this.searchTimeout) clearTimeout(this.searchTimeout)
    this.globalSearch.clear()
  }
}
