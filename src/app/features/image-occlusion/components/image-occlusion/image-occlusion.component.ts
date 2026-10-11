import { Component, HostListener, OnInit, signal } from '@angular/core'
import { FormsModule } from '@angular/forms'
import { DecimalPipe } from '@angular/common'
import { invoke } from '@tauri-apps/api/core'
import { TranslocoPipe } from '@ngneat/transloco'
import { ImageOcclusionCard, ImageOcclusionSummary, OcclusionMask } from '../../models/image-occlusion.model'

@Component({
  selector: 'app-image-occlusion',
  standalone: true,
  imports: [FormsModule, DecimalPipe, TranslocoPipe],
  templateUrl: './image-occlusion.component.html',
  styleUrl: './image-occlusion.component.scss',
})
export class ImageOcclusionComponent implements OnInit {
  readonly cards = signal<ImageOcclusionSummary[]>([])
  readonly loadedCard = signal<ImageOcclusionCard | null>(null)
  readonly imageData = signal<string | null>(null)
  readonly imageWidth = signal(1)
  readonly imageHeight = signal(1)
  readonly masks = signal<OcclusionMask[]>([])
  readonly activeCardId = signal<string | null>(null)
  readonly revealedMasks = signal<number[]>([])
  readonly draftMask = signal<OcclusionMask | null>(null)
  readonly isDrawing = signal(false)
  readonly isSaving = signal(false)
  readonly isAnswering = signal(false)
  readonly errorMessage = signal<string | null>(null)
  readonly activeTab = signal<'create' | 'review'>('create')
  private dragStart = { x: 0, y: 0 }

  get activeCard(): ImageOcclusionCard | undefined {
    return this.loadedCard() ?? undefined
  }

  get activeImage(): string | null {
    return this.activeTab() === 'create' ? this.imageData() : (this.activeCard?.imageData ?? null)
  }

  get currentMasks(): OcclusionMask[] {
    return this.activeTab() === 'create' ? this.masks() : (this.activeCard?.masks ?? [])
  }

  get imageAspectRatio(): string {
    return `${this.imageWidth()} / ${this.imageHeight()}`
  }

  async ngOnInit(): Promise<void> {
    await this.loadCards()
  }

  async loadCards(): Promise<void> {
    try {
      this.cards.set(await invoke<ImageOcclusionSummary[]>('list_image_occlusion_cards'))
    } catch (error) {
      this.errorMessage.set(String(error))
    }
  }

  async onFileSelected(event: Event): Promise<void> {
    const input = event.target as HTMLInputElement
    const file = input.files?.[0]
    if (file) await this.readImage(file)
    input.value = ''
  }

  @HostListener('window:paste', ['$event'])
  async onPaste(event: ClipboardEvent): Promise<void> {
    const file = Array.from(event.clipboardData?.items ?? [])
      .find((item) => item.type.startsWith('image/'))
      ?.getAsFile()
    if (file) {
      event.preventDefault()
      await this.readImage(file)
    }
  }

  private async readImage(file: File): Promise<void> {
    if (!['image/png', 'image/jpeg', 'image/webp'].includes(file.type)) {
      this.errorMessage.set('imageOcclusion.errors.imageType')
      return
    }
    if (file.size > 15 * 1024 * 1024) {
      this.errorMessage.set('imageOcclusion.errors.imageSize')
      return
    }
    const reader = new FileReader()
    reader.onload = () => {
      const image = new Image()
      image.onload = () => {
        this.imageData.set(String(reader.result))
        this.imageWidth.set(image.naturalWidth)
        this.imageHeight.set(image.naturalHeight)
        this.masks.set([])
        this.errorMessage.set(null)
      }
      image.src = String(reader.result)
    }
    reader.readAsDataURL(file)
  }

  startDrawing(event: PointerEvent): void {
    if (!this.imageData() || this.activeTab() !== 'create' || event.button !== 0) return
    const point = this.normalizedPoint(event)
    this.dragStart = point
    this.isDrawing.set(true)
    this.draftMask.set({ x: point.x, y: point.y, width: 0, height: 0 })
    ;(event.currentTarget as SVGSVGElement).setPointerCapture(event.pointerId)
  }

  continueDrawing(event: PointerEvent): void {
    if (!this.isDrawing()) return
    this.draftMask.set(this.maskFromPoint(this.normalizedPoint(event)))
  }

  finishDrawing(event: PointerEvent): void {
    if (!this.isDrawing()) return
    const mask = this.maskFromPoint(this.normalizedPoint(event))
    if (mask.width > 0.01 && mask.height > 0.01) this.masks.update((existing) => [...existing, mask])
    this.isDrawing.set(false)
    this.draftMask.set(null)
  }

  removeMask(index: number): void {
    this.masks.update((existing) => existing.filter((_, maskIndex) => maskIndex !== index))
  }

  revealMask(index: number): void {
    this.revealedMasks.update((revealed) => revealed.includes(index) ? revealed : [...revealed, index])
  }

  async selectCard(card: ImageOcclusionSummary): Promise<void> {
    this.activeCardId.set(card.id)
    this.revealedMasks.set([])
    this.loadedCard.set(null)
    try {
      const loadedCard = await invoke<ImageOcclusionCard>('get_image_occlusion_card', { cardId: card.id })
      if (this.activeCardId() === card.id) this.loadedCard.set(loadedCard)
    } catch (error) {
      this.errorMessage.set(String(error))
    }
  }

  setTab(tab: 'create' | 'review'): void {
    this.activeTab.set(tab)
    this.errorMessage.set(null)
    if (tab === 'review' && !this.activeCardId() && this.cards().length) void this.selectCard(this.cards()[0])
  }

  async saveCard(): Promise<void> {
    const imageData = this.imageData()
    if (!imageData || !this.masks().length || this.isSaving()) return
    this.isSaving.set(true)
    this.errorMessage.set(null)
    try {
      const id = await invoke<string>('save_image_occlusion_card', {
        imageData,
        masks: this.masks(),
        deckId: null,
      })
      await this.loadCards()
      this.imageData.set(null)
      this.masks.set([])
      const savedSummary = this.cards().find((card) => card.id === id)
      this.activeTab.set('review')
      if (savedSummary) await this.selectCard(savedSummary)
    } catch (error) {
      this.errorMessage.set(String(error))
    } finally {
      this.isSaving.set(false)
    }
  }

  async answerCard(rating: number): Promise<void> {
    const cardId = this.activeCardId()
    const card = this.activeCard
    if (!cardId || !card || this.revealedMasks().length !== card.masks.length || this.isAnswering()) return
    this.isAnswering.set(true)
    try {
      await invoke('answer_image_occlusion_card', { cardId, rating })
      this.activeCardId.set(null)
      this.loadedCard.set(null)
      this.revealedMasks.set([])
      await this.loadCards()
      if (this.cards().length) await this.selectCard(this.cards()[0])
    } catch (error) {
      this.errorMessage.set(String(error))
    } finally {
      this.isAnswering.set(false)
    }
  }

  clearEditor(): void {
    this.imageData.set(null)
    this.masks.set([])
    this.errorMessage.set(null)
  }

  private normalizedPoint(event: PointerEvent): { x: number; y: number } {
    const bounds = (event.currentTarget as SVGSVGElement).getBoundingClientRect()
    return {
      x: Math.max(0, Math.min(1, (event.clientX - bounds.left) / bounds.width)),
      y: Math.max(0, Math.min(1, (event.clientY - bounds.top) / bounds.height)),
    }
  }

  private maskFromPoint(point: { x: number; y: number }): OcclusionMask {
    return {
      x: Math.min(this.dragStart.x, point.x),
      y: Math.min(this.dragStart.y, point.y),
      width: Math.abs(point.x - this.dragStart.x),
      height: Math.abs(point.y - this.dragStart.y),
    }
  }
}

