import {
  AfterViewInit,
  ChangeDetectionStrategy,
  Component,
  ElementRef,
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
import { AnalysisMode, MorphologicalToken, NotebookEntry } from '../../models/notebook.model'
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
  private readonly translocoService = inject(TranslocoService, { optional: true })

  @ViewChild('editorElement') editorElementRef?: ElementRef<HTMLDivElement>
  editor: Editor | null = null

  readonly analysisMode = signal<AnalysisMode>('manual')
  readonly currentId = signal<string>(crypto.randomUUID())
  readonly title = signal<string>('Minha Anotação')
  readonly markdownContent = signal<string>('私は学生です。\n毎日日本語を勉強します。')
  readonly tokens = signal<MorphologicalToken[]>([])
  readonly toastMessage = signal<string | null>(null)
  private toastTimeout: ReturnType<typeof setTimeout> | null = null

  readonly formattedHtml = computed<string>(() => {
    return this.notebookService.formatTokensToHtml(this.tokens())
  })

  // RxJS pipeline for 750ms debounce in auto mode
  private readonly textChange$ = new Subject<string>()
  private textSubscription: Subscription | null = null

  // Guard flag to prevent infinite loops when setContent updates TipTap
  private isApplyingGrammarHighlight = false

  async ngOnInit(): Promise<void> {
    await this.notebookService.loadEntries()
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
          placeholder: 'Comece a digitar em japonês ou anote seus estudos (estilo Obsidian)...',
        }),
      ],
      content: `<p>${this.markdownContent().replace(/\n/g, '</p><p>')}</p>`,
      onUpdate: ({ editor }) => {
        if (this.isApplyingGrammarHighlight) {
          return
        }

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

  onTitleChange(event: Event): void {
    const val = (event.target as HTMLInputElement).value
    this.title.set(val)
  }

  selectEntry(entry: NotebookEntry): void {
    this.currentId.set(entry.id)
    this.title.set(entry.title)
    this.markdownContent.set(entry.content_md)

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
        currentText
      )

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
