import {
  ApplicationRef,
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  EnvironmentInjector,
  HostListener,
  OnDestroy,
  OnInit,
  ViewChild,
  computed,
  inject,
  signal,
} from '@angular/core'
import { ActivatedRoute, Router } from '@angular/router'
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
import {
  AnalysisMode,
  MorphologicalToken,
  NotebookEntry,
  SlashMenuItem,
  SyntaxChunk,
} from '../../models/notebook.model'
import { SafeHtmlPipe } from '../../../../shared/pipes/safe-html.pipe'
import { TranslocoDirective, TranslocoService } from '@ngneat/transloco'
import { SyntaxTreeExtension } from '../../extensions/syntax-tree.extension'
import { SyntaxNodeComponent } from '../syntax-node/syntax-node.component'
import { BidirectionalLinkExtension } from '../../extensions/bidirectional-link.extension'
import {
  getGrammarLintText,
  GrammarLinterExtension,
  setGrammarLintDecorations,
} from '../../extensions/grammar-linter.extension'
import { LintDecoration, LintTooltipState } from '../../models/notebook.model'
import { notebookTextToEditorHtml } from '../../utils/notebook-link-html'

@Component({
  selector: 'app-notebook-editor',
  standalone: true,
  imports: [SafeHtmlPipe, TranslocoDirective, SyntaxNodeComponent],
  templateUrl: './notebook-editor.component.html',
  styleUrl: './notebook-editor.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NotebookEditorComponent implements OnInit, OnDestroy {
  readonly notebookService = inject(NotebookService)
  readonly deckService = inject(DeckService)
  private readonly router = inject(Router)
  private readonly activatedRoute = inject(ActivatedRoute, { optional: true })
  private readonly translocoService = inject(TranslocoService, { optional: true })
  private readonly injector = inject(EnvironmentInjector)
  private readonly appRef = inject(ApplicationRef)
  private trayNewNoteSubscription: Subscription | null = null
  private queryParamsSubscription: Subscription | null = null
  private grammarLintTimeout: ReturnType<typeof setTimeout> | null = null
  private grammarLintRequestId = 0
  private lintTooltipTimeout: ReturnType<typeof setTimeout> | null = null

  private editorElement?: ElementRef<HTMLDivElement>
  @ViewChild('editorElement')
  set editorElementRef(element: ElementRef<HTMLDivElement> | undefined) {
    this.editorElement = element
    if (!element) {
      if (this.editor) {
        this.editorHtml.set(this.editor.getHTML())
        this.markdownContent.set(this.editor.getText())
        this.editor.destroy()
        this.editor = null
      }
      return
    }
    this.initializeEditor(element.nativeElement)
  }
  editor: Editor | null = null

  readonly analysisMode = signal<AnalysisMode>('manual')
  readonly currentId = signal<string>(crypto.randomUUID())
  readonly selectedDeckId = signal<string | null>(this.deckService.targetDeckId())
  readonly notebookSearch = signal<string>('')
  readonly filteredEntries = computed(() => {
    const query = this.notebookSearch().trim().toLocaleLowerCase()
    if (!query) return this.notebookService.entries()
    return this.notebookService
      .entries()
      .filter((entry) => `${entry.title} ${entry.content_md}`.toLocaleLowerCase().includes(query))
  })
  readonly title = signal<string>('Minha Anotação')
  readonly markdownContent = signal<string>('私は学生です。\n毎日日本語を勉強します。')
  private readonly editorHtml = signal<string>(notebookTextToEditorHtml(this.markdownContent()))
  readonly tokens = signal<MorphologicalToken[]>([])
  readonly isTreeView = signal<boolean>(false)
  readonly syntaxTreeData = signal<SyntaxChunk[]>([])
  readonly toastMessage = signal<string | null>(null)
  private toastTimeout: ReturnType<typeof setTimeout> | null = null

  toggleSyntaxTree(): void {
    this.isTreeView.update((v) => !v)
  }

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
    {
      id: 'syntax-tree',
      titleKey: 'notebook.slash_menu.syntax_tree_title',
      descKey: 'notebook.slash_menu.syntax_tree_desc',
      icon: 'pi pi-sitemap',
      action: () => this.executeSlashAction(() => void this.insertSyntaxTree()),
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

  // Selection Bubble Menu state
  readonly showBubbleMenu = signal<boolean>(false)
  readonly bubbleMenuPosition = signal<{ top: number; left: number }>({ top: 0, left: 0 })
  readonly selectedText = signal<string>('')
  readonly activeLintTooltip = signal<LintTooltipState | null>(null)

  async ngOnInit(): Promise<void> {
    this.queryParamsSubscription =
      this.activatedRoute?.queryParamMap.subscribe((params) => {
        this.notebookSearch.set(params.get('search') ?? '')
      }) ?? null
    this.trayNewNoteSubscription = this.notebookService.newNoteRequests.subscribe(() => {
      this.notebookService.consumeNewNoteRequest()
      this.createNewNote()
    })
    await Promise.all([this.notebookService.loadEntries(), this.deckService.loadDecks()])
    const linkedNoteId = this.activatedRoute?.snapshot.queryParamMap.get('noteId')
    const linkedNote = this.notebookService.entries().find((entry) => entry.id === linkedNoteId)
    if (linkedNote) this.selectEntry(linkedNote)
    if (this.notebookService.newNoteRequested()) {
      this.notebookService.consumeNewNoteRequest()
      this.createNewNote()
    }
    await this.triggerAnalysis(this.markdownContent())

    // Configure RxJS debounceTime(750) for auto mode
    this.textSubscription = this.textChange$.pipe(debounceTime(750)).subscribe((text) => {
      if (this.analysisMode() === 'auto') {
        void this.performMorphologicalAnalysis(text)
      }
    })
  }

  private initializeEditor(element: HTMLDivElement): void {
    if (this.editor) return
    this.editor = new Editor({
      element,
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
          placeholder: "Digite '/' para blocos e comandos, ou anote em japonês...",
        }),
        SyntaxTreeExtension.configure({
          injector: this.injector,
          appRef: this.appRef,
        }),
        BidirectionalLinkExtension.configure({
          onOpenLink: (title) => this.openLinkedNote(title),
        }),
        GrammarLinterExtension.configure({
          onHover: (lint, coordinates) => this.onGrammarLintHover(lint, coordinates),
        }),
      ],
      content: this.editorHtml(),
      onSelectionUpdate: ({ editor }) => {
        this.updateBubbleMenu(editor)
      },
      onUpdate: ({ editor }) => {
        if (this.isApplyingGrammarHighlight) {
          return
        }

        this.checkSlashCommand(editor)
        this.updateBubbleMenu(editor)

        const text = editor.getText()
        this.markdownContent.set(text)
        this.editorHtml.set(editor.getHTML())
        this.scheduleGrammarLint(getGrammarLintText(editor))

        if (this.analysisMode() === 'auto') {
          this.textChange$.next(text)
        }
      },
    })
    if (this.editor) this.scheduleGrammarLint(getGrammarLintText(this.editor))
  }

  ngOnDestroy(): void {
    this.trayNewNoteSubscription?.unsubscribe()
    this.queryParamsSubscription?.unsubscribe()
    if (this.toastTimeout) {
      clearTimeout(this.toastTimeout)
    }
    this.textSubscription?.unsubscribe()
    if (this.grammarLintTimeout) clearTimeout(this.grammarLintTimeout)
    if (this.lintTooltipTimeout) clearTimeout(this.lintTooltipTimeout)
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
    const [tokens, syntaxChunks] = await Promise.all([
      this.notebookService.analyzeText(rawText),
      this.notebookService.generateSyntaxTree(rawText),
    ])
    this.tokens.set(tokens)
    this.syntaxTreeData.set(syntaxChunks)
  }

  /**
   * Performs morphological analysis and syntax tree generation via Rust backend,
   * updating TipTap with Tailwind color spans while preserving exact cursor coordinates.
   */
  async performMorphologicalAnalysis(text: string): Promise<void> {
    if (!text || !text.trim()) {
      this.tokens.set([])
      this.syntaxTreeData.set([])
      return
    }

    const [tokens, syntaxChunks] = await Promise.all([
      this.notebookService.analyzeText(text),
      this.notebookService.generateSyntaxTree(text),
    ])
    this.tokens.set(tokens)
    this.syntaxTreeData.set(syntaxChunks)

    if (!this.editor) return
    let hasBidirectionalLinks = false
    this.editor.state.doc.descendants((node) => {
      if (node.type.name === 'bidirectionalLink') hasBidirectionalLinks = true
    })
    if (hasBidirectionalLinks) return

    const htmlContent = this.notebookService.formatTokensToHtml(tokens)
    const wrappedHtml = `<p>${htmlContent.replace(/\n/g, '</p><p>')}</p>`

    // Preserve cursor / selection anchor and head positions
    const { from, to } = this.editor.state.selection

    this.isApplyingGrammarHighlight = true
    try {
      this.editor.commands.setContent(wrappedHtml, {
        emitUpdate: false,
      })
      this.editorHtml.set(this.editor.getHTML())

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
      const containerRect = this.editorElement?.nativeElement.getBoundingClientRect()
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

  updateBubbleMenu(editor: Editor): void {
    const { from, to } = editor.state.selection
    if (from !== to) {
      const selected = editor.state.doc.textBetween(from, to, '\n', '\0').trim()
      if (selected.length > 0) {
        this.selectedText.set(selected)
        const coords = editor.view.coordsAtPos(from)
        const containerRect = this.editorElement?.nativeElement.getBoundingClientRect()
        if (containerRect) {
          this.bubbleMenuPosition.set({
            top: Math.max(0, coords.top - containerRect.top - 46),
            left: Math.max(0, coords.left - containerRect.left),
          })
          this.showBubbleMenu.set(true)
          return
        }
      }
    }
    this.showBubbleMenu.set(false)
  }

  async insertSyntaxTree(customText?: string): Promise<void> {
    if (!this.editor) return

    let targetText = customText
    if (!targetText) {
      const { from, to } = this.editor.state.selection
      if (from !== to) {
        targetText = this.editor.state.doc.textBetween(from, to, '\n', '\0').trim()
      } else {
        // Fallback to active paragraph or sentence
        const text = this.editor.getText().trim()
        targetText = text.split('\n')[0] || '私は学生です'
      }
    }

    if (!targetText) {
      targetText = '私は学生です'
    }

    try {
      const chunks = await this.notebookService.generateSyntaxTree(targetText)
      if (chunks && chunks.length > 0) {
        this.editor
          .chain()
          .focus()
          .insertContent({
            type: 'syntaxTree',
            attrs: {
              chunks: JSON.stringify(chunks),
            },
          })
          .run()
        this.showBubbleMenu.set(false)
      }
    } catch (err) {
      console.error('Falha ao inserir nó de árvore sintática:', err)
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
      this.selectedSlashIndex.update(
        (i) => (i - 1 + this.slashMenuItems.length) % this.slashMenuItems.length,
      )
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
    if (val) this.deckService.setTargetDeck(val)
  }

  private scheduleGrammarLint(text: string): void {
    if (this.grammarLintTimeout) clearTimeout(this.grammarLintTimeout)
    const requestId = ++this.grammarLintRequestId
    this.activeLintTooltip.set(null)
    if (!text.trim()) {
      if (this.editor) setGrammarLintDecorations(this.editor, [])
      return
    }

    this.grammarLintTimeout = setTimeout(() => {
      void this.notebookService.lintJapaneseText(text).then((suggestions) => {
        if (
          requestId === this.grammarLintRequestId &&
          this.editor &&
          getGrammarLintText(this.editor) === text
        ) {
          setGrammarLintDecorations(this.editor, suggestions)
        }
      })
    }, 500)
  }

  private onGrammarLintHover(
    lint: LintDecoration | null,
    coordinates: { x: number; y: number },
  ): void {
    if (this.lintTooltipTimeout) clearTimeout(this.lintTooltipTimeout)
    if (!lint) {
      this.lintTooltipTimeout = setTimeout(() => this.activeLintTooltip.set(null), 180)
      return
    }
    this.activeLintTooltip.set({
      ...lint,
      clientX: coordinates.x,
      clientY: coordinates.y,
    })
  }

  cancelLintTooltipDismiss(): void {
    if (this.lintTooltipTimeout) clearTimeout(this.lintTooltipTimeout)
    this.lintTooltipTimeout = null
  }

  dismissLintTooltip(): void {
    if (this.lintTooltipTimeout) clearTimeout(this.lintTooltipTimeout)
    this.lintTooltipTimeout = setTimeout(() => this.activeLintTooltip.set(null), 180)
  }

  lintTooltipLeft(clientX: number): number {
    return Math.max(8, Math.min(clientX + 12, window.innerWidth - 300))
  }

  lintTooltipTop(clientY: number): number {
    return Math.max(8, Math.min(clientY + 12, window.innerHeight - 150))
  }

  applyLintSuggestion(): void {
    const lint = this.activeLintTooltip()
    if (!lint || !this.editor) return
    this.editor
      .chain()
      .focus()
      .insertContentAt({ from: lint.from, to: lint.to }, lint.suggestion.suggestion)
      .run()
    this.activeLintTooltip.set(null)
  }

  selectEntry(entry: NotebookEntry): void {
    this.currentId.set(entry.id)
    this.title.set(entry.title)
    this.markdownContent.set(entry.content_md)
    this.editorHtml.set(notebookTextToEditorHtml(entry.content_md))
    this.selectedDeckId.set(entry.deck_id ?? null)

    if (this.editor) {
      this.editor.commands.setContent(notebookTextToEditorHtml(entry.content_md))
      this.scheduleGrammarLint(getGrammarLintText(this.editor))
    }
    void this.triggerAnalysis(entry.content_md)
  }

  openLinkedNote(title: string): void {
    const normalizedTitle = title.trim().toLocaleLowerCase()
    if (!normalizedTitle) return

    const entry = this.notebookService
      .entries()
      .find((candidate) => candidate.title.trim().toLocaleLowerCase() === normalizedTitle)

    if (entry) {
      this.selectEntry(entry)
      void this.router.navigate(['/notebook'], {
        queryParams: { noteId: entry.id, search: null },
      })
      return
    }

    this.notebookSearch.set(title)
    void this.router.navigate(['/notebook'], {
      queryParams: { search: title, noteId: null },
    })
  }

  createNewNote(): void {
    const newId = crypto.randomUUID()
    this.currentId.set(newId)
    const defaultTitle =
      this.translocoService?.translate('notebook.default_new_note') ?? 'Nova Anotação'
    this.title.set(defaultTitle)
    this.markdownContent.set('')
    this.editorHtml.set('<p></p>')
    this.selectedDeckId.set(this.deckService.targetDeckId())

    if (this.editor) {
      this.editor.commands.setContent('<p></p>')
    }
    this.tokens.set([])
  }

  async saveNote(): Promise<void> {
    try {
      const currentText = this.editor?.getText() || this.markdownContent()
      let currentTitle = this.title().trim()

      const defaultNew =
        this.translocoService?.translate('notebook.default_new_note') ?? 'Nova Anotação'
      const untitled =
        this.translocoService?.translate('notebook.untitled_note') ?? 'Nota sem título'

      // If title is default or empty, extract from first line of content if available
      if (
        !currentTitle ||
        currentTitle === defaultNew ||
        currentTitle === 'Nova Anotação' ||
        currentTitle === 'Minha Anotação'
      ) {
        const firstLine = currentText
          .split('\n')
          .map((l) => l.trim())
          .find((l) => l.length > 0)
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
        0,
      )

      // Guarantee immediate reactive reload of notes list
      await this.notebookService.loadEntries()
      const now = new Date().toISOString()
      const existingEntry = this.notebookService
        .entries()
        .find((entry) => entry.id === this.currentId())
      this.notebookService.upsertEntry({
        id: this.currentId(),
        title: currentTitle || untitled,
        content_md: currentText,
        created_at: existingEntry?.created_at ?? now,
        updated_at: now,
        deck_id: this.selectedDeckId(),
        next_review_date: existingEntry?.next_review_date ?? null,
        srs_stage: existingEntry?.srs_stage ?? 0,
      })

      const successMsg =
        this.translocoService?.translate('notebook.save_success') ?? 'Nota guardada com sucesso!'
      this.showToast(successMsg)
    } catch (err) {
      console.error('Falha ao guardar nota:', err)
      const errorMsg =
        this.translocoService?.translate('notebook.save_error') ??
        'Erro ao guardar nota. Tente novamente.'
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
