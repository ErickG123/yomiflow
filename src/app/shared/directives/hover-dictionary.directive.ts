import { Directive, HostListener, OnDestroy, inject } from '@angular/core'
import { ContextualDictionaryService } from '../../features/dictionary/services/contextual-dictionary.service'
import { extractJapaneseWord } from '../utils/japanese-word-extraction'

@Directive({
  selector: '[appHoverDictionary]',
  standalone: true,
})
export class HoverDictionaryDirective implements OnDestroy {
  private readonly contextualDictionary = inject(ContextualDictionaryService)
  private readonly caretDocument = document
  private debounceTimer?: ReturnType<typeof setTimeout>
  private pendingWord: string | null = null

  @HostListener('mousemove', ['$event'])
  onMouseMove(event: MouseEvent): void {
    if (!event.shiftKey) {
      this.cancelPendingLookup()
      this.contextualDictionary.hide()
      return
    }

    const word = this.getWordAtPoint(event.clientX, event.clientY)
    if (!word) {
      this.cancelPendingLookup()
      this.contextualDictionary.hide()
      return
    }

    this.contextualDictionary.moveTo(event.clientX, event.clientY)
    if (word === this.pendingWord || word === this.contextualDictionary.word()) return

    this.cancelPendingLookup()
    this.pendingWord = word
    this.debounceTimer = setTimeout(() => {
      this.debounceTimer = undefined
      void this.contextualDictionary.lookup(word, event.clientX, event.clientY)
    }, 300)
  }

  @HostListener('window:keyup', ['$event'])
  onKeyUp(event: KeyboardEvent): void {
    if (event.key === 'Shift') {
      this.cancelPendingLookup()
      this.contextualDictionary.hide()
    }
  }

  @HostListener('mouseleave')
  onMouseLeave(): void {
    this.cancelPendingLookup()
    this.contextualDictionary.hide()
  }

  ngOnDestroy(): void {
    this.cancelPendingLookup()
    this.contextualDictionary.hide()
  }

  private getWordAtPoint(x: number, y: number): string | null {
    const range = this.caretDocument.caretRangeFromPoint?.(x, y)
    if (range?.startContainer.nodeType === Node.TEXT_NODE) {
      return extractJapaneseWord(range.startContainer.textContent ?? '', range.startOffset)
    }

    const caret = this.caretDocument.caretPositionFromPoint?.(x, y)
    if (caret?.offsetNode.nodeType === Node.TEXT_NODE) {
      return extractJapaneseWord(caret.offsetNode.textContent ?? '', caret.offset)
    }
    return null
  }

  private cancelPendingLookup(): void {
    if (this.debounceTimer) clearTimeout(this.debounceTimer)
    this.debounceTimer = undefined
    this.pendingWord = null
  }
}
