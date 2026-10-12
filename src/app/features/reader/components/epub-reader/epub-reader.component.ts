import {
  AfterViewInit,
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  HostListener,
  OnDestroy,
  OnInit,
  SecurityContext,
  ViewChild,
  inject,
  signal,
} from '@angular/core'
import { CommonModule } from '@angular/common'
import { DomSanitizer } from '@angular/platform-browser'
import { TranslocoDirective } from '@ngneat/transloco'
import { EpubReaderService } from '../../services/epub-reader.service'
import { DictionaryService } from '../../../dictionary/services/dictionary.service'
import { DictionaryEntry } from '../../../dictionary/models/dictionary.model'
import { PitchAccentComponent } from '../../../../shared/components/pitch-accent/pitch-accent.component'
import { DeckService } from '../../../decks/services/deck.service'
import { NotebookService } from '../../../notebook/services/notebook.service'
import { EpubBookInstance, EpubRenditionInstance } from '../../models/epub.model'
import ePub from 'epubjs'

@Component({
  selector: 'app-epub-reader',
  standalone: true,
  imports: [CommonModule, TranslocoDirective, PitchAccentComponent],
  templateUrl: './epub-reader.component.html',
  styleUrl: './epub-reader.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class EpubReaderComponent implements OnInit, AfterViewInit, OnDestroy {
  private readonly sanitizer = inject(DomSanitizer)
  readonly epubService = inject(EpubReaderService)
  readonly dictionaryService = inject(DictionaryService)
  readonly deckService = inject(DeckService)
  readonly notebookService = inject(NotebookService)

  @ViewChild('epubViewer') viewerContainerRef?: ElementRef<HTMLDivElement>

  private book: EpubBookInstance | null = null
  private rendition: EpubRenditionInstance | null = null

  readonly bookTitle = signal<string>('Light Novel')
  readonly isBookLoaded = signal<boolean>(false)
  readonly isFuriganaEnabled = signal<boolean>(true)
  readonly activeWord = signal<string | null>(null)
  readonly activeWordResults = signal<DictionaryEntry[]>([])
  readonly isSearchingWord = signal<boolean>(false)
  readonly toastMessage = signal<string | null>(null)
  readonly selectedDeckId = signal<string>('default')

  ngOnInit(): void {
    void this.deckService.loadDecks()
  }

  ngAfterViewInit(): void {
    if (this.epubService.epubBase64Data()) {
      this.initEpubBook(this.epubService.epubBase64Data()!)
    }
  }

  ngOnDestroy(): void {
    this.rendition?.destroy?.()
    this.book?.destroy?.()
  }

  async openFile(): Promise<void> {
    const filePath = await this.epubService.openEpubDialog()
    if (filePath && this.epubService.epubBase64Data()) {
      this.initEpubBook(this.epubService.epubBase64Data()!)
    }
  }

  private currentBookHash = ''

  private computeBookHash(pathOrData: string): string {
    let hash = 0
    for (let i = 0; i < pathOrData.length; i++) {
      hash = (hash << 5) - hash + pathOrData.charCodeAt(i)
      hash |= 0
    }
    return `epub-${Math.abs(hash).toString(16)}`
  }

  private async initEpubBook(dataUri: string): Promise<void> {
    const container = this.viewerContainerRef?.nativeElement
    if (!container) {
      return
    }

    this.rendition?.destroy?.()
    this.book?.destroy?.()

    try {
      this.currentBookHash = this.computeBookHash(
        this.epubService.currentEpubPath() || dataUri.slice(0, 500),
      )
      this.book = ePub(dataUri)

      this.book?.loaded?.metadata?.then((meta) => {
        if (meta?.title) {
          this.bookTitle.set(meta.title)
        }
      })

      this.rendition = this.book?.renderTo?.(container, {
        width: '100%',
        height: '100%',
        flow: 'paginated',
        manager: 'default',
        spread: 'auto',
      }) as typeof this.rendition

      // Hook rendition hooks to dynamically process chapter DOM with Lindera Furigana
      this.rendition?.hooks?.content?.register?.(async (contents) => {
        const doc = contents.document
        if (!doc || !doc.body) {
          return
        }

        // Apply dark reading theme to EPUB contents
        const head = doc.head || doc.getElementsByTagName('head')[0]
        const style = doc.createElement('style')
        style.textContent = `
          body {
            color: #e4e4e7 !important;
            background-color: transparent !important;
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", "Hiragino Sans", "Hiragino Kaku Gothic ProN", Meiryo, sans-serif !important;
            line-height: 2.1 !important;
            font-size: 1.15rem !important;
          }
          ruby rt {
            font-size: 0.6em !important;
            color: #a1a1aa !important;
            user-select: none !important;
          }
          a { color: #818cf8 !important; }
        `
        head.appendChild(style)

        if (this.isFuriganaEnabled()) {
          const bodyHtml = doc.body.innerHTML
          const transformed = await this.epubService.injectFurigana(bodyHtml)
          doc.body.innerHTML = this.sanitizer.sanitize(SecurityContext.HTML, transformed) ?? ''
        }

        // Add word click lookup inside iframe
        doc.body.addEventListener('click', (e: MouseEvent) => {
          const target = e.target as HTMLElement
          if (target && target.tagName.toLowerCase() === 'ruby') {
            const text = target.innerText.replace(/\s+/g, '')
            void this.lookupWord(text)
          } else {
            const selection = doc.getSelection()?.toString().trim()
            if (selection) {
              void this.lookupWord(selection)
            }
          }
        })
      })

      // Listen for page relocation and persist CFI in SQLite
      this.rendition?.on?.('relocated', (location: unknown) => {
        const loc = location as { start?: { cfi?: string } } | undefined
        const cfi = loc?.start?.cfi
        if (cfi && this.currentBookHash) {
          void this.epubService.saveReadingPosition(this.currentBookHash, cfi)
        }
      })

      // Restore saved location if present, otherwise start at beginning
      const savedCfi = await this.epubService.getReadingPosition(this.currentBookHash)
      if (savedCfi) {
        this.rendition?.display?.(savedCfi)
      } else {
        this.rendition?.display?.()
      }

      this.isBookLoaded.set(true)
    } catch (err) {
      console.error('Failed to initialize EPUB rendition:', err)
    }
  }

  nextPage(): void {
    this.rendition?.next?.()
  }

  prevPage(): void {
    this.rendition?.prev?.()
  }

  toggleFurigana(): void {
    this.isFuriganaEnabled.update((v) => !v)
    this.rendition?.display?.()
  }

  async lookupWord(word: string): Promise<void> {
    const cleanWord = word.trim()
    if (!cleanWord) {
      return
    }

    this.activeWord.set(cleanWord)
    this.isSearchingWord.set(true)
    try {
      const results = await this.dictionaryService.searchDictionary(cleanWord)
      this.activeWordResults.set(results)
    } catch (err) {
      console.error('Error looking up word in EPUB:', err)
      this.activeWordResults.set([])
    } finally {
      this.isSearchingWord.set(false)
    }
  }

  closeDictionary(): void {
    this.activeWord.set(null)
    this.activeWordResults.set([])
  }

  async addWordToSrs(entry: DictionaryEntry): Promise<void> {
    try {
      if (entry.id > 0) {
        await this.dictionaryService.addToSrs(entry.id, this.selectedDeckId())
      } else {
        await this.dictionaryService.addCustomWordToSrs(
          entry.kanji,
          entry.reading,
          entry.meaning,
          this.selectedDeckId(),
        )
      }
      this.showToast('Palavra adicionada ao SRS!')
      this.activeWordResults.update((list) =>
        list.map((item) => (item === entry ? { ...item, in_srs: true } : item)),
      )
    } catch (err) {
      console.error('Erro ao adicionar palavra ao SRS:', err)
      this.showToast('Falha ao adicionar palavra ao SRS')
    }
  }

  async sendToNotebook(entry?: DictionaryEntry): Promise<void> {
    const word = this.activeWord() || this.bookTitle()
    const def = entry ? `${entry.reading} - ${entry.meaning}` : ''
    const content = `### ${word}\n${def ? `> **Leitura / Significado:** ${def}\n\n` : ''}**Origem:** [Light Novel EPUB] ${this.bookTitle()}`

    try {
      const noteId = `epub-${Date.now()}`
      await this.notebookService.saveEntry(noteId, `[LN] ${word}`, content, this.selectedDeckId())
      this.showToast('Nota salva no Caderno!')
    } catch (err) {
      console.error('Erro ao salvar no caderno:', err)
      this.showToast('Falha ao salvar no Caderno')
    }
  }

  private showToast(msg: string): void {
    this.toastMessage.set(msg)
    setTimeout(() => {
      if (this.toastMessage() === msg) {
        this.toastMessage.set(null)
      }
    }, 3000)
  }

  @HostListener('window:keydown', ['$event'])
  handleKeyboardEvent(event: KeyboardEvent): void {
    if (event.key === 'ArrowRight' || event.key === 'd' || event.key === 'D') {
      this.nextPage()
    } else if (event.key === 'ArrowLeft' || event.key === 'a' || event.key === 'A') {
      this.prevPage()
    }
  }
}
