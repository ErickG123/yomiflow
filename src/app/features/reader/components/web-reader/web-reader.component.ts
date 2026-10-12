import { ChangeDetectionStrategy, Component, OnInit, computed, inject, signal } from '@angular/core'
import { CommonModule } from '@angular/common'
import { TranslocoDirective } from '@ngneat/transloco'
import { NewsService } from '../../services/news.service'
import { NhkNewsItem, TokenSpan } from '../../models/news.model'
import { DictionaryService } from '../../../dictionary/services/dictionary.service'
import { DictionaryEntry } from '../../../dictionary/models/dictionary.model'
import { NotebookService } from '../../../notebook/services/notebook.service'
import { PitchAccentComponent } from '../../../../shared/components/pitch-accent/pitch-accent.component'
import { DeckService } from '../../../decks/services/deck.service'
import { HoverDictionaryDirective } from '../../../../shared/directives/hover-dictionary.directive'
import { FuriganaPipe } from '../../../../shared/pipes/furigana.pipe'

@Component({
  selector: 'app-web-reader',
  standalone: true,
  imports: [
    CommonModule,
    TranslocoDirective,
    PitchAccentComponent,
    HoverDictionaryDirective,
    FuriganaPipe,
  ],
  templateUrl: './web-reader.component.html',
  styleUrl: './web-reader.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class WebReaderComponent implements OnInit {
  readonly newsService = inject(NewsService)
  readonly dictionaryService = inject(DictionaryService)
  readonly notebookService = inject(NotebookService)
  readonly deckService = inject(DeckService)

  readonly selectedDeckId = signal<string>('default')
  readonly activeWord = signal<string | null>(null)
  readonly activeWordResults = signal<DictionaryEntry[]>([])
  readonly isSearchingWord = signal<boolean>(false)
  readonly toastMessage = signal<string | null>(null)

  // Tokenize article body into interactable word units
  readonly tokenizedParagraphs = computed<TokenSpan[][]>(() => {
    const article = this.newsService.selectedArticle()
    if (!article || !article.body) {
      return []
    }

    const paragraphs = article.body.split(/\r?\n+/).filter((p) => p.trim().length > 0)
    return paragraphs.map((para) => this.tokenizeSentence(para))
  })

  ngOnInit(): void {
    void this.newsService.fetchNews()
    void this.deckService.loadDecks()
  }

  tokenizeSentence(sentence: string): TokenSpan[] {
    const tokens: TokenSpan[] = []
    // Match Japanese sequences (Kanji, Hiragana, Katakana) or other segments
    const regex =
      /([\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}ー]+|[^\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}ー]+)/gu
    let match: RegExpExecArray | null
    while ((match = regex.exec(sentence)) !== null) {
      const text = match[0]
      const isWord = /[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}]/u.test(text)
      tokens.push({ text, isWord })
    }
    return tokens
  }

  selectArticle(article: NhkNewsItem): void {
    this.newsService.selectArticle(article)
    this.closeHoverDictionary()
  }

  async onWordClick(word: string): Promise<void> {
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
      console.error('Failed to search word definition:', err)
      this.activeWordResults.set([])
    } finally {
      this.isSearchingWord.set(false)
    }
  }

  closeHoverDictionary(): void {
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
    const article = this.newsService.selectedArticle()
    const word = this.activeWord() || (article ? article.title : 'NHK Easy News')
    const definition = entry ? `${entry.reading} - ${entry.meaning}` : ''
    const example = entry?.example_sentence ? `> **Exemplo:** ${entry.example_sentence}\n\n` : ''
    const content = `### ${word}\n${definition ? `> **Leitura/Significado:** ${definition}\n\n` : ''}${example}${article ? `**Artigo:** [${article.title}](${article.news_web_url || '#'})\n\n${article.body}` : ''}`

    try {
      const noteId = `news-${Date.now()}`
      await this.notebookService.saveEntry(noteId, `[NHK] ${word}`, content, this.selectedDeckId())
      this.showToast('Nota enviada para o Caderno!')
    } catch (err) {
      console.error('Erro ao enviar para o Caderno:', err)
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
}
