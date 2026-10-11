import {
  AfterViewInit,
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  HostListener,
  OnDestroy,
  OnInit,
  ViewChild,
  computed,
  inject,
  signal,
} from '@angular/core'
import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import { Table } from '@tiptap/extension-table'
import TableRow from '@tiptap/extension-table-row'
import TableCell from '@tiptap/extension-table-cell'
import TableHeader from '@tiptap/extension-table-header'
import Placeholder from '@tiptap/extension-placeholder'
import { Subject, Subscription } from 'rxjs'
import { debounceTime } from 'rxjs/operators'
import { NotebookService } from '../../services/notebook.service'
import { DeckService } from '../../../decks/services/deck.service'
import { AnalysisMode, MorphologicalToken, NotebookEntry, SlashMenuItem } from '../../models/notebook.model'
import { SafeHtmlPipe } from '../../../../shared/pipes/safe-html.pipe'
import { TranslocoDirective, TranslocoService } from '@ngneat/transloco'

@Component({
  selector: 'app-notebook-editor',
  standalone: true,
  imports: [SafeHtmlPipe, TranslocoDirective],
  templateUrl: './notebook-editor.component.html',
  styleUrl: './notebook-editor.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NotebookEditorComponent implements OnInit, AfterViewInit, OnDestroy {
  readonly notebookService = inject(NotebookService)
  readonly deckService = inject(DeckService)
  private readonly translocoService = inject(TranslocoService, { optional: true })

  @ViewChild('editorElement') editorElementRef?: ElementRef<HTMLDivElement>
  editor: Editor | null = null

  readonly analysisMode = signal<AnalysisMode>('manual')
  readonly currentId = signal<string>(crypto.randomUUID())
  readonly selectedDeckId = signal<string | null>(null)
  readonly title = signal<string>('Minha Anotação')
  readonly markdownContent = signal<string>('私は学生です。\n毎日日本語を勉強します。')
  readonly tokens = signal<MorphologicalToken[]>([])
  readonly toastMessage = signal<string | null>(null)
  private toastTimeout: ReturnType<typeof setTimeout> | null = null

  // Slash Commands state
  readonly showSlashMenu = signal<boolean>(false)
  readonly slashMenuPosition = signal<{ top: number; left: number }>({ top: 0, left: 0 })
  readonly selectedSlashIndex = signal<number>(0)

  readonly slashMenuItems: SlashMenuItem[] = [
    {
      id: 'h1',
      titleKey: 'notebook.slash_menu.h1_title',
      descKey: 'notebook.slash_menu.h1_desc',
      icon: 'H1',
      action: () => this.executeSlashAction(() => this.toggleHeading(1)),
    },
    {
      id: 'h2',
      titleKey: 'notebook.slash_menu.h2_title',
      descKey: 'notebook.slash_menu.h2_desc',
      icon: 'H2',
      action: () => this.executeSlashAction(() => this.toggleHeading(2)),
    },
    {
      id: 'h3',
      titleKey: 'notebook.slash_menu.h3_title',
      descKey: 'notebook.slash_menu.h3_desc',
      icon: 'H3',
      action: () => this.executeSlashAction(() => this.toggleHeading(3)),
    },
    {
      id: 'bullet',
      titleKey: 'notebook.slash_menu.bullet_title',
      descKey: 'notebook.slash_menu.bullet_desc',
      icon: 'pi pi-list',
      action: () => this.executeSlashAction(() => this.toggleBulletList()),
    },
    {
      id: 'ordered',
      titleKey: 'notebook.slash_menu.ordered_title',
      descKey: 'notebook.slash_menu.ordered_desc',
      icon: 'pi pi-sort-numeric-down',
      action: () => this.executeSlashAction(() => this.toggleOrderedList()),
    },
    {
      id: 'quote',
      titleKey: 'notebook.slash_menu.quote_title',
      descKey: 'notebook.slash_menu.quote_desc',
      icon: 'pi pi-comment',
      action: () => this.executeSlashAction(() => this.toggleBlockquote()),
    },
    {
      id: 'table',
      titleKey: 'notebook.slash_menu.table_title',
      descKey: 'notebook.slash_menu.table_desc',
      icon: 'pi pi-table',
      action: () => this.executeSlashAction(() => this.insertTable()),
    },
  ]

  readonly formattedHtml = computed<string>(() => {
    return this.notebookService.formatTokensToHtml(this.tokens())
  })

  // RxJS pipeline for 750ms debounce in auto mode
  private readonly textChange$ = new Subject<string>()
  private textSubscription: Subscription | null = null

  // Guard flag to prevent infinite loops when setContent updates TipTap
  private isApplyingGrammarHighlight = false

  async ngOnInit(): Promise<void> {
    await Promise.all([
      this.notebookService.loadEntries(),
      this.deckService.loadDecks(),
    ])
    await this.triggerAnalysis(this.markdownContent())

    // Configure RxJS debounceTime(750) for auto mode
    this.textSubscription = this.textChange$
      .pipe(debounceTime(750))
      .subscribe((text) => {
        if (this.analysisMode() === 'auto') {
          void this.performMorphologicalAnalysis(text)
        }
      })
  }

  ngAfterViewInit(): void {
    if (!this.editorElementRef) return

    this.editor = new Editor({
      element: this.editorElementRef.nativeElement,
      extensions: [
        StarterKit.configure({
          heading: {
            levels: [1, 2, 3],
          },
        }),
        Table.configure({
          resizable: true,
        }),
        TableRow,
        TableHeader,
        TableCell,
        Placeholder.configure({
          placeholder: 'Digite \'/\' para blocos e comandos, ou anote em japonês...',
        }),
      ],
      content: `<p>${this.markdownContent().replace(/\n/g, '</p><p>')}</p>`,
      onUpdate: ({ editor }) => {
        if (this.isApplyingGrammarHighlight) {
          return
        }

        this.checkSlashCommand(editor)

        const text = editor.getText()
        this.markdownContent.set(text)

        if (this.analysisMode() === 'auto') {
          this.textChange$.next(text)
        }
      },
    })
  }

  ngOnDestroy(): void {
    if (this.toastTimeout) {
      clearTimeout(this.toastTimeout)
    }
    this.textSubscription?.unsubscribe()
    this.editor?.destroy()
  }

  // --- TipTap Formatting Action Helpers ---
  toggleBold(): void {
    this.editor?.chain().focus().toggleBold().run()
  }

  toggleItalic(): void {
    this.editor?.chain().focus().toggleItalic().run()
  }

  toggleHeading(level: 1 | 2 | 3): void {
    this.editor?.chain().focus().toggleHeading({ level }).run()
  }

  toggleBulletList(): void {
    this.editor?.chain().focus().toggleBulletList().run()
  }

  toggleOrderedList(): void {
    this.editor?.chain().focus().toggleOrderedList().run()
  }

  toggleBlockquote(): void {
    this.editor?.chain().focus().toggleBlockquote().run()
  }

  insertTable(): void {
    this.editor?.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run()
  }

  setAnalysisMode(mode: AnalysisMode): void {
    this.analysisMode.set(mode)
    if (mode === 'auto') {
      const text = this.editor?.getText() || this.markdownContent()
      this.textChange$.next(text)
    }
  }

  async analyzeManually(): Promise<void> {
    const text = this.editor?.getText() || this.markdownContent()
    await this.performMorphologicalAnalysis(text)
  }

  async triggerAnalysis(rawText: string): Promise<void> {
    const tokens = await this.notebookService.analyzeText(rawText)
    this.tokens.set(tokens)
  }

  /**
   * Performs morphological analysis via Rust backend and updates TipTap with Tailwind color spans
   * preserving exact cursor/selection coordinates via editor.state.selection and emitUpdate: false.
   */
  async performMorphologicalAnalysis(text: string): Promise<void> {
    if (!text || !text.trim()) {
      this.tokens.set([])
      return
    }

    const tokens = await this.notebookService.analyzeText(text)
    this.tokens.set(tokens)

    if (!this.editor) return

    const htmlContent = this.notebookService.formatTokensToHtml(tokens)
    const wrappedHtml = `<p>${htmlContent.replace(/\n/g, '</p><p>')}</p>`

    // Preserve cursor / selection anchor and head positions
    const { from, to } = this.editor.state.selection

    this.isApplyingGrammarHighlight = true
    try {
      this.editor.commands.setContent(wrappedHtml, {
        emitUpdate: false,
      })

      // Safely restore selection within the newly rendered document bounds
      const docSize = this.editor.state.doc.content.size
      const safeFrom = Math.min(from, docSize)
      const safeTo = Math.min(to, docSize)

      this.editor.commands.setTextSelection({ from: safeFrom, to: safeTo })
    } finally {
      this.isApplyingGrammarHighlight = false
    }
  }

  checkSlashCommand(editor: Editor): void {
    const { from } = editor.state.selection
    const textBefore = editor.state.doc.textBetween(Math.max(0, from - 1), from, '\n', '\0')
    if (textBefore === '/') {
      const coords = editor.view.coordsAtPos(from)
      const containerRect = this.editorElementRef?.nativeElement.getBoundingClientRect()
      if (containerRect) {
        this.slashMenuPosition.set({
          top: coords.bottom - containerRect.top + 8,
          left: Math.max(0, coords.left - containerRect.left),
        })
      }
      this.selectedSlashIndex.set(0)
      this.showSlashMenu.set(true)
    } else if (this.showSlashMenu()) {
      this.showSlashMenu.set(false)
    }
  }

  executeSlashAction(action: () => void): void {
    if (!this.editor) return
    const { from } = this.editor.state.selection
    // Remove the trailing '/' character typed by the user
    const textBefore = this.editor.state.doc.textBetween(Math.max(0, from - 1), from, '\n', '\0')
    if (textBefore === '/') {
      this.editor.commands.deleteRange({ from: from - 1, to: from })
    }
    this.showSlashMenu.set(false)
    action()
  }

  @HostListener('window:keydown', ['$event'])
  handleKeyDown(event: KeyboardEvent): void {
    if (!this.showSlashMenu()) return

    if (event.key === 'ArrowDown') {
      event.preventDefault()
      this.selectedSlashIndex.update((i) => (i + 1) % this.slashMenuItems.length)
    } else if (event.key === 'ArrowUp') {
      event.preventDefault()
      this.selectedSlashIndex.update((i) => (i - 1 + this.slashMenuItems.length) % this.slashMenuItems.length)
    } else if (event.key === 'Enter') {
      event.preventDefault()
      const item = this.slashMenuItems[this.selectedSlashIndex()]
      if (item) {
        item.action()
      }
    } else if (event.key === 'Escape') {
      this.showSlashMenu.set(false)
    }
  }

  onTitleChange(event: Event): void {
    const val = (event.target as HTMLInputElement).value
    this.title.set(val)
  }

  onDeckChange(event: Event): void {
    const val = (event.target as HTMLSelectElement).value
    this.selectedDeckId.set(val ? val : null)
  }

  selectEntry(entry: NotebookEntry): void {
    this.currentId.set(entry.id)
    this.title.set(entry.title)
    this.markdownContent.set(entry.content_md)
    this.selectedDeckId.set(entry.deck_id ?? null)

    if (this.editor) {
      this.editor.commands.setContent(`<p>${entry.content_md.replace(/\n/g, '</p><p>')}</p>`)
    }
    void this.triggerAnalysis(entry.content_md)
  }

  createNewNote(): void {
    const newId = crypto.randomUUID()
    this.currentId.set(newId)
    const defaultTitle = this.translocoService?.translate('notebook.default_new_note') ?? 'Nova Anotação'
    this.title.set(defaultTitle)
    this.markdownContent.set('')
    this.selectedDeckId.set(null)

    if (this.editor) {
      this.editor.commands.setContent('<p></p>')
    }
    this.tokens.set([])
  }

  async saveNote(): Promise<void> {
    try {
      const currentText = this.editor?.getText() || this.markdownContent()
      let currentTitle = this.title().trim()

      const defaultNew = this.translocoService?.translate('notebook.default_new_note') ?? 'Nova Anotação'
      const untitled = this.translocoService?.translate('notebook.untitled_note') ?? 'Nota sem título'

      // If title is default or empty, extract from first line of content if available
      if (!currentTitle || currentTitle === defaultNew || currentTitle === 'Nova Anotação' || currentTitle === 'Minha Anotação') {
        const firstLine = currentText.split('\n').map((l) => l.trim()).find((l) => l.length > 0)
        if (firstLine) {
          currentTitle = firstLine.slice(0, 50)
          this.title.set(currentTitle)
        }
      }

      await this.notebookService.saveEntry(
        this.currentId(),
        currentTitle || untitled,
        currentText,
        this.selectedDeckId(),
        null,
        0
      )

      // Guarantee immediate reactive reload of notes list
      await this.notebookService.loadEntries()

      const successMsg = this.translocoService?.translate('notebook.save_success') ?? 'Nota guardada com sucesso!'
      this.showToast(successMsg)
    } catch (err) {
      console.error('Falha ao guardar nota:', err)
      const errorMsg = this.translocoService?.translate('notebook.save_error') ?? 'Erro ao guardar nota. Tente novamente.'
      this.showToast(errorMsg)
    }
  }

  private showToast(message: string): void {
    if (this.toastTimeout) {
      clearTimeout(this.toastTimeout)
    }
    this.toastMessage.set(message)
    this.toastTimeout = setTimeout(() => {
      this.toastMessage.set(null)
    }, 3000)
  }
}
