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
import { Subject, Subscription } from 'rxjs'
import { debounceTime } from 'rxjs/operators'
import { NotebookService } from '../../services/notebook.service'
import { AnalysisMode, MorphologicalToken, NotebookEntry } from '../../models/notebook.model'
import { SafeHtmlPipe } from '../../../../shared/pipes/safe-html.pipe'

@Component({
  selector: 'app-notebook-editor',
  standalone: true,
  imports: [SafeHtmlPipe],
  templateUrl: './notebook-editor.component.html',
  styleUrl: './notebook-editor.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class NotebookEditorComponent implements OnInit, AfterViewInit, OnDestroy {
  readonly notebookService = inject(NotebookService)

  @ViewChild('editorElement') editorElementRef?: ElementRef<HTMLDivElement>
  private editor: Editor | null = null

  readonly analysisMode = signal<AnalysisMode>('manual')
  readonly currentId = signal<string>(crypto.randomUUID())
  readonly title = signal<string>('Minha Anotação')
  readonly markdownContent = signal<string>('私は学生です。\n毎日日本語を勉強します。')
  readonly tokens = signal<MorphologicalToken[]>([])

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
      extensions: [StarterKit],
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
    this.textSubscription?.unsubscribe()
    this.editor?.destroy()
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
    this.title.set('Nova Anotação')
    this.markdownContent.set('')

    if (this.editor) {
      this.editor.commands.setContent('<p></p>')
    }
    this.tokens.set([])
  }

  async saveNote(): Promise<void> {
    await this.notebookService.saveEntry(
      this.currentId(),
      this.title(),
      this.markdownContent()
    )
  }
}
