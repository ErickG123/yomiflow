import {
  AfterViewInit,
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  Input,
  OnChanges,
  OnInit,
  SimpleChanges,
  ViewChild,
  inject,
  signal,
} from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { DictionaryService } from '../../../features/dictionary/services/dictionary.service'
import { SafeHtmlPipe } from '../../pipes/safe-html.pipe'
import { StrokeCoordinate } from '../../models/media.model'

@Component({
  selector: 'app-kanji-pad',
  standalone: true,
  imports: [SafeHtmlPipe],
  templateUrl: './kanji-pad.component.html',
  styleUrl: './kanji-pad.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class KanjiPadComponent implements OnInit, AfterViewInit, OnChanges {
  @Input() kanji!: string

  @ViewChild('drawingCanvas') canvasRef?: ElementRef<HTMLCanvasElement>

  private readonly dictionaryService = inject(DictionaryService)
  readonly svgContent = signal<string | null>(null)

  // Strokes record: Array of Array of points
  readonly userStrokes = signal<StrokeCoordinate[][]>([])
  private currentStroke: StrokeCoordinate[] = []

  // Evaluation feedback state: 'idle' | 'success' | 'fail'
  readonly evaluationStatus = signal<'idle' | 'evaluating' | 'success' | 'fail'>('idle')
  readonly evaluationScore = signal<number | null>(null)

  private ctx: CanvasRenderingContext2D | null = null
  private isDrawing = false

  ngOnInit(): void {
    void this.loadKanjiSvg()
  }

  ngAfterViewInit(): void {
    this.setupCanvas()
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['kanji'] && !changes['kanji'].firstChange) {
      this.clear()
      void this.loadKanjiSvg()
    }
  }

  async loadKanjiSvg(): Promise<void> {
    if (!this.kanji) {
      this.svgContent.set(null)
      return
    }

    try {
      const svg = await this.dictionaryService.getKanjiSvg(this.kanji)
      this.svgContent.set(svg)
    } catch (err) {
      console.warn(`Failed to load SVG for kanji ${this.kanji}:`, err)
      this.svgContent.set(null)
    }
  }

  private setupCanvas(): void {
    const canvas = this.canvasRef?.nativeElement
    if (!canvas) {
      return
    }

    // Canvas coordinate space matches 256x256 container
    canvas.width = 256
    canvas.height = 256

    const context = canvas.getContext('2d')
    if (context) {
      context.lineWidth = 6
      context.lineCap = 'round'
      context.lineJoin = 'round'
      context.strokeStyle = '#e4e4e7' // zinc-200
      this.ctx = context
    }
  }

  onMouseDown(event: MouseEvent): void {
    if (!this.ctx) {
      return
    }
    this.isDrawing = true
    const { x, y } = this.getCanvasCoords(event)
    this.currentStroke = [{ x, y }]
    this.ctx.beginPath()
    this.ctx.moveTo(x, y)
  }

  onMouseMove(event: MouseEvent): void {
    if (!this.isDrawing || !this.ctx) {
      return
    }
    const { x, y } = this.getCanvasCoords(event)
    this.currentStroke.push({ x, y })
    this.ctx.lineTo(x, y)
    this.ctx.stroke()
  }

  onMouseUp(): void {
    this.stopDrawing()
  }

  onMouseLeave(): void {
    this.stopDrawing()
  }

  private stopDrawing(): void {
    if (this.isDrawing && this.ctx) {
      this.ctx.closePath()
      if (this.currentStroke.length > 0) {
        this.userStrokes.update((strokes) => [...strokes, [...this.currentStroke]])
        this.currentStroke = []
      }
    }
    this.isDrawing = false
  }

  private getCanvasCoords(event: MouseEvent): { x: number; y: number } {
    const canvas = this.canvasRef?.nativeElement
    if (!canvas) {
      return { x: 0, y: 0 }
    }
    const rect = canvas.getBoundingClientRect()
    return {
      x: event.clientX - rect.left,
      y: event.clientY - rect.top,
    }
  }

  async evaluateDrawing(): Promise<void> {
    const strokes = this.userStrokes()
    if (!this.kanji || strokes.length === 0) {
      return
    }

    this.evaluationStatus.set('evaluating')
    try {
      const score = await invoke<number>('evaluate_kanji_drawing', {
        kanji: this.kanji,
        userStrokes: strokes,
      })

      this.evaluationScore.set(score)

      const strokeColor = score >= 0.8 ? '#10b981' : score >= 0.5 ? '#eab308' : '#f43f5e'
      this.redrawStrokesWithColor(strokeColor)

      if (score >= 0.8) {
        this.evaluationStatus.set('success')
      } else {
        this.evaluationStatus.set('fail')
      }
    } catch (err) {
      console.error('Error evaluating kanji:', err)
      this.evaluationStatus.set('idle')
    }
  }

  private redrawStrokesWithColor(color: string): void {
    const canvas = this.canvasRef?.nativeElement
    if (!canvas || !this.ctx) return

    this.ctx.clearRect(0, 0, canvas.width, canvas.height)
    this.ctx.strokeStyle = color
    this.ctx.lineWidth = 5
    this.ctx.lineCap = 'round'
    this.ctx.lineJoin = 'round'

    const strokes = this.userStrokes()
    for (const stroke of strokes) {
      if (stroke.length === 0) continue
      this.ctx.beginPath()
      this.ctx.moveTo(stroke[0].x, stroke[0].y)
      for (let i = 1; i < stroke.length; i++) {
        this.ctx.lineTo(stroke[i].x, stroke[i].y)
      }
      this.ctx.stroke()
    }
    this.ctx.closePath()
    // Reset back to normal stroke color for future strokes
    this.ctx.strokeStyle = '#e4e4e7'
  }

  clear(): void {
    const canvas = this.canvasRef?.nativeElement
    if (canvas && this.ctx) {
      this.ctx.clearRect(0, 0, canvas.width, canvas.height)
    }
    this.userStrokes.set([])
    this.currentStroke = []
    this.evaluationStatus.set('idle')
    this.evaluationScore.set(null)
  }
}
