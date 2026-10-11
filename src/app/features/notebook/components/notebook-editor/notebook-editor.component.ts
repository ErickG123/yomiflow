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
import { NotebookService } from '../../services/notebook.service'
import { MorphologicalToken, NotebookEntry } from '../../models/notebook.model'
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

  readonly currentId = signal<string>(crypto.randomUUID())
  readonly title = signal<string>('Minha Anotação')
  readonly markdownContent = signal<string>('私は学生です。\n毎日日本語を勉強します。')
  readonly tokens = signal<MorphologicalToken[]>([])

  readonly formattedHtml = computed<string>(() => {
    return this.notebookService.formatTokensToHtml(this.tokens())
  })

  private analyzeDebounceTimeout: ReturnType<typeof setTimeout> | null = null

  async ngOnInit(): Promise<void> {
    await this.notebookService.loadEntries()
    await this.triggerAnalysis(this.markdownContent())
  }

  ngAfterViewInit(): void {
    if (!this.editorElementRef) return

    this.editor = new Editor({
      element: this.editorElementRef.nativeElement,
      extensions: [StarterKit],
      content: `<p>${this.markdownContent().replace(/\n/g, '</p><p>')}</p>`,
      onUpdate: ({ editor }) => {
        const text = editor.getText()
        this.markdownContent.set(text)

        if (this.analyzeDebounceTimeout) {
          clearTimeout(this.analyzeDebounceTimeout)
        }
        this.analyzeDebounceTimeout = setTimeout(() => {
          void this.triggerAnalysis(text)
        }, 300)
      },
    })
  }

  ngOnDestroy(): void {
    if (this.analyzeDebounceTimeout) {
      clearTimeout(this.analyzeDebounceTimeout)
    }
    this.editor?.destroy()
  }

  async triggerAnalysis(rawText: string): Promise<void> {
    const tokens = await this.notebookService.analyzeText(rawText)
    this.tokens.set(tokens)
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
