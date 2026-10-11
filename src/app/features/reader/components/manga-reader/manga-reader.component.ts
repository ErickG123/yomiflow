import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  HostListener,
  OnInit,
  ViewChild,
  inject,
  signal,
} from '@angular/core'
import { Router } from '@angular/router'
import { TranslocoPipe } from '@ngneat/transloco'
import { ReaderService } from '../../services/reader.service'
import { DictionaryService } from '../../../dictionary/services/dictionary.service'
import { DictionaryEntry } from '../../../dictionary/models/dictionary.model'
import { invoke } from '@tauri-apps/api/core'
import { AudioService } from '../../../../core/services/audio.service'
import { SelectionRect } from '../../models/manga.model'
import { AiModelService } from '../../services/ai-model.service'
import { sanitizeJapaneseText } from '../../../../shared/utils/text-utils'
import { PitchAccentComponent } from '../../../../shared/components/pitch-accent/pitch-accent.component'

@Component({
  selector: 'app-manga-reader',
  standalone: true,
  imports: [TranslocoPipe, PitchAccentComponent],
  templateUrl: './manga-reader.component.html',
  styleUrl: './manga-reader.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class MangaReaderComponent implements OnInit {
  private readonly router = inject(Router)
  readonly readerService = inject(ReaderService)
  readonly dictionaryService = inject(DictionaryService)
  readonly audioService = inject(AudioService)
  readonly aiService = inject(AiModelService)

  readonly currentCbzPath = this.readerService.currentCbzPath
  readonly pages = this.readerService.pages
  readonly currentIndex = this.readerService.currentIndex
  readonly currentPageBase64 = this.readerService.currentPageBase64
  readonly isLoadingPage = this.readerService.isLoadingPage

  // OCR state
  readonly isExtractingText = signal<boolean>(false)
  readonly extractedText = signal<string | null>(null)
  readonly isDrawerOpen = signal<boolean>(false)
  readonly dictionaryResults = this.dictionaryService.searchResults
  readonly kanjiResults = this.dictionaryService.kanjiSearchResults
  readonly isSearchingDictionary = this.dictionaryService.isLoading

  // Canvas and Selection
  @ViewChild('pageImage') pageImageRef?: ElementRef<HTMLImageElement>
  @ViewChild('overlayCanvas') overlayCanvasRef?: ElementRef<HTMLCanvasElement>

  private selection: SelectionRect | null = null
  private isSelecting = false
  private isDragging = false

  ngOnInit(): void {
    // If no manga is loaded when directly navigating to reader, redirect to library
    if (!this.currentCbzPath()) {
      void this.router.navigate(['/manga'])
      return
    }

    // Automatically check and download MangaOCR model on first use
    void this.aiService.ensureModelExists()
  }

  backToLibrary(): void {
    void this.router.navigate(['/manga'])
  }

  async openManga(): Promise<void> {
    await this.readerService.openCbzFileDialog()
  }

  closeDrawer(): void {
    this.isDrawerOpen.set(false)
    // Return focus to body so keyboard shortcuts resume immediately without focus trap
    if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur()
    }
    document.body.focus()
  }

  async openDeepL(): Promise<void> {
    const text = this.extractedText()
    if (!text) {
      return
    }
    const url = `https://www.deepl.com/translator#ja/pt/${encodeURIComponent(text)}`
    try {
      const { openUrl } = await import('@tauri-apps/plugin-opener')
      await openUrl(url)
    } catch {
      window.open(url, '_blank')
    }
  }

  async openGoogleTranslate(): Promise<void> {
    const text = this.extractedText()
    if (!text) {
      return
    }
    const url = `https://translate.google.com/?sl=ja&tl=pt&text=${encodeURIComponent(text)}`
    try {
      const { openUrl } = await import('@tauri-apps/plugin-opener')
      await openUrl(url)
    } catch {
      window.open(url, '_blank')
    }
  }

  // --- MOUSE SELECTION / CROP EVENTS ---

  onMouseDown(event: MouseEvent): void {
    if (this.aiService.isDownloading()) {
      return
    }

    const canvas = this.overlayCanvasRef?.nativeElement
    if (!canvas) {
      return
    }

    const rect = canvas.getBoundingClientRect()
    const x = event.clientX - rect.left
    const y = event.clientY - rect.top

    this.isSelecting = true
    this.isDragging = false
    this.selection = {
      startX: x,
      startY: y,
      currentX: x,
      currentY: y,
    }

    this.drawSelection(canvas)
  }

  onMouseMove(event: MouseEvent): void {
    if (!this.isSelecting || !this.selection) {
      return
    }

    const canvas = this.overlayCanvasRef?.nativeElement
    if (!canvas) {
      return
    }

    const rect = canvas.getBoundingClientRect()
    this.selection.currentX = event.clientX - rect.left
    this.selection.currentY = event.clientY - rect.top

    const dx = Math.abs(this.selection.currentX - this.selection.startX)
    const dy = Math.abs(this.selection.currentY - this.selection.startY)
    if (dx > 4 || dy > 4) {
      this.isDragging = true
    }

    this.drawSelection(canvas)
  }

  async onMouseUp(event: MouseEvent): Promise<void> {
    if (!this.isSelecting) {
      return
    }
    this.isSelecting = false

    const canvas = this.overlayCanvasRef?.nativeElement
    const img = this.pageImageRef?.nativeElement
    if (!canvas || !img || !this.selection) {
      this.clearCanvas(canvas)
      return
    }

    const sel = this.selection
    this.selection = null

    const selWidth = Math.abs(sel.currentX - sel.startX)
    const selHeight = Math.abs(sel.currentY - sel.startY)

    // If it was just a quick click without dragging, treat as page navigation!
    if (!this.isDragging || selWidth < 10 || selHeight < 10) {
      this.clearCanvas(canvas)
      this.handlePageNavigationClick(event, canvas)
      return
    }

    // Surgical Crop: Calculate bounding box in canvas coordinates
    const cropX = Math.min(sel.startX, sel.currentX)
    const cropY = Math.min(sel.startY, sel.currentY)

    // Compute scale between rendered canvas and intrinsic natural image dimensions
    const scaleX = img.naturalWidth / canvas.width
    const scaleY = img.naturalHeight / canvas.height

    const sourceX = cropX * scaleX
    const sourceY = cropY * scaleY
    const sourceWidth = selWidth * scaleX
    const sourceHeight = selHeight * scaleY

    // Offscreen canvas for cropping
    const cropCanvas = document.createElement('canvas')
    cropCanvas.width = Math.max(1, Math.round(sourceWidth))
    cropCanvas.height = Math.max(1, Math.round(sourceHeight))
    const cropCtx = cropCanvas.getContext('2d')

    if (cropCtx) {
      cropCtx.drawImage(
        img,
        sourceX,
        sourceY,
        sourceWidth,
        sourceHeight,
        0,
        0,
        cropCanvas.width,
        cropCanvas.height
      )

      const croppedBase64 = cropCanvas.toDataURL('image/png')
      this.clearCanvas(canvas)
      await this.performOcr(croppedBase64)
    } else {
      this.clearCanvas(canvas)
    }
  }

  private handlePageNavigationClick(event: MouseEvent, canvas: HTMLCanvasElement): void {
    const rect = canvas.getBoundingClientRect()
    const clickX = event.clientX - rect.left
    const width = rect.width

    // Manga RTL: left half advances, right half goes back
    if (clickX < width / 2) {
      this.readerService.nextPage()
    } else {
      this.readerService.prevPage()
    }
  }

  private drawSelection(canvas: HTMLCanvasElement): void {
    const ctx = canvas.getContext('2d')
    if (!ctx) {
      return
    }

    ctx.clearRect(0, 0, canvas.width, canvas.height)

    if (!this.selection || !this.isDragging) {
      return
    }

    const sel = this.selection
    const x = Math.min(sel.startX, sel.currentX)
    const y = Math.min(sel.startY, sel.currentY)
    const w = Math.abs(sel.currentX - sel.startX)
    const h = Math.abs(sel.currentY - sel.startY)

    // Dim the outside region
    ctx.fillStyle = 'rgba(0, 0, 0, 0.45)'
    ctx.fillRect(0, 0, canvas.width, canvas.height)

    // Clear the selected box
    ctx.clearRect(x, y, w, h)

    // Vibrant Orange border
    ctx.strokeStyle = '#f97316'
    ctx.lineWidth = 2
    ctx.strokeRect(x, y, w, h)
  }

  private clearCanvas(canvas?: HTMLCanvasElement): void {
    if (!canvas) {
      return
    }
    const ctx = canvas.getContext('2d')
    if (ctx) {
      ctx.clearRect(0, 0, canvas.width, canvas.height)
    }
  }

  onImageLoaded(): void {
    const img = this.pageImageRef?.nativeElement
    const canvas = this.overlayCanvasRef?.nativeElement
    if (img && canvas) {
      canvas.width = img.clientWidth
      canvas.height = img.clientHeight
    }
  }

  async performOcr(croppedBase64: string): Promise<void> {
    this.isExtractingText.set(true)
    try {
      let raw: string
      if (this.aiService.isModelReady()) {
        try {
          raw = await invoke<string>('process_image_with_ai', {
            base64Image: croppedBase64,
          })
        } catch (aiErr) {
          console.warn('AI OCR falhou, tentando fallback para Tesseract OCR:', aiErr)
          const fallbackRaw = await this.readerService.processOcr(croppedBase64)
          raw = sanitizeJapaneseText(fallbackRaw)
        }
      } else {
        const tesseractRaw = await this.readerService.processOcr(croppedBase64)
        raw = sanitizeJapaneseText(tesseractRaw)
      }

      const sanitized = raw ? raw.trim() : ''

      if (sanitized.length > 0) {
        this.extractedText.set(sanitized)
        this.isDrawerOpen.set(true)
        // Query the dictionary immediately with extracted text
        await this.dictionaryService.search(sanitized)
      } else {
        alert('Nenhum texto em japonês reconhecido no recorte.')
      }
    } catch (error) {
      console.error('OCR Error:', error)
      alert(`Erro no OCR: ${error}`)
    } finally {
      this.isExtractingText.set(false)
    }
  }

  async toggleSrs(entry: DictionaryEntry): Promise<void> {
    if (entry.in_srs) {
      await this.dictionaryService.removeFromSrs(entry.id)
    } else {
      await this.dictionaryService.addToSrs(entry.id)
    }
  }

  @HostListener('window:keydown', ['$event'])
  handleKeyboardEvent(event: KeyboardEvent): void {
    const activeTag = document.activeElement?.tagName.toLowerCase()
    if (activeTag === 'input' || activeTag === 'textarea') {
      return
    }

    if (!this.currentCbzPath()) {
      return
    }

    // Escape closes drawer if open, or returns to library if drawer is closed
    if (event.key === 'Escape') {
      event.preventDefault()
      if (this.isDrawerOpen()) {
        this.closeDrawer()
      } else {
        this.backToLibrary()
      }
      return
    }

    if (this.isDrawerOpen()) {
      return
    }

    // Japanese reading: Left goes Forward, Right goes Backward
    if (event.key === 'ArrowLeft' || event.key === 'a' || event.key === 'A') {
      event.preventDefault()
      void this.readerService.nextPage()
    } else if (event.key === 'ArrowRight' || event.key === 'd' || event.key === 'D') {
      event.preventDefault()
      void this.readerService.prevPage()
    }
  }
}
