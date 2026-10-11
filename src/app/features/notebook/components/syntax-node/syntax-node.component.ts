import {
  AfterViewInit,
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  HostListener,
  Input,
  OnChanges,
  OnDestroy,
  QueryList,
  SimpleChanges,
  ViewChild,
  ViewChildren,
  afterNextRender,
  signal,
} from '@angular/core'
import { CommonModule } from '@angular/common'
import { BunsetsuChunk } from '../../models/notebook.model'
import { SyntaxArc } from '../../models/syntax-tree.model'

@Component({
  selector: 'app-syntax-node',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './syntax-node.component.html',
  styleUrl: './syntax-node.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class SyntaxNodeComponent implements AfterViewInit, OnChanges, OnDestroy {
  @Input() set data(val: BunsetsuChunk[] | string | null | undefined) {
    if (typeof val === 'string') {
      try {
        const parsed = JSON.parse(val) as BunsetsuChunk[]
        this.chunks.set(Array.isArray(parsed) ? parsed : [])
      } catch {
        this.chunks.set([])
      }
    } else if (Array.isArray(val)) {
      this.chunks.set(val)
    } else {
      this.chunks.set([])
    }
  }

  @ViewChild('container') containerRef?: ElementRef<HTMLDivElement>
  @ViewChildren('chunkElement') chunkElements!: QueryList<ElementRef<HTMLDivElement>>

  readonly chunks = signal<BunsetsuChunk[]>([])
  readonly arcs = signal<SyntaxArc[]>([])
  readonly svgWidth = signal<number>(600)
  readonly svgHeight = signal<number>(200)

  private resizeObserver: ResizeObserver | null = null

  constructor() {
    afterNextRender(() => {
      this.recalculateArcs()
    })
  }

  ngAfterViewInit(): void {
    if (this.containerRef?.nativeElement && typeof ResizeObserver !== 'undefined') {
      this.resizeObserver = new ResizeObserver(() => {
        this.recalculateArcs()
      })
      this.resizeObserver.observe(this.containerRef.nativeElement)
    }

    this.recalculateArcs()
  }

  ngOnChanges(changes: SimpleChanges): void {
    if (changes['data']) {
      setTimeout(() => {
        this.recalculateArcs()
      }, 50)
    }
  }

  ngOnDestroy(): void {
    this.resizeObserver?.disconnect()
  }

  @HostListener('window:resize')
  onWindowResize(): void {
    this.recalculateArcs()
  }

  recalculateArcs(): void {
    const container = this.containerRef?.nativeElement
    if (!container || !this.chunkElements || this.chunkElements.length === 0) {
      this.arcs.set([])
      return
    }

    const containerRect = container.getBoundingClientRect()
    this.svgWidth.set(container.scrollWidth || containerRect.width)
    this.svgHeight.set(container.scrollHeight || containerRect.height)

    const elementMap = new Map<number, HTMLElement>()
    this.chunkElements.forEach((elRef) => {
      const el = elRef.nativeElement
      const idAttr = el.getAttribute('data-chunk-id')
      if (idAttr) {
        elementMap.set(parseInt(idAttr, 10), el)
      }
    })

    const newArcs: SyntaxArc[] = []
    const chunksList = this.chunks()

    chunksList.forEach((chunk) => {
      if (chunk.target_id === null || chunk.target_id === undefined) {
        return
      }

      const sourceEl = elementMap.get(chunk.id)
      const targetEl = elementMap.get(chunk.target_id)
      if (!sourceEl || !targetEl) {
        return
      }

      const sourceRect = sourceEl.getBoundingClientRect()
      const targetRect = targetEl.getBoundingClientRect()

      // Calculate anchor points relative to container's content area
      const x1 = sourceRect.left - containerRect.left + sourceRect.width / 2 + container.scrollLeft
      const y1 = sourceRect.top - containerRect.top + container.scrollTop

      const x2 = targetRect.left - containerRect.left + targetRect.width / 2 + container.scrollLeft
      const y2 = targetRect.top - containerRect.top + container.scrollTop

      // Height of the quadratic curve arch based on horizontal distance
      const distance = Math.abs(x2 - x1)
      const archHeight = Math.min(80, Math.max(35, distance * 0.35))
      const cy = Math.max(10, Math.min(y1, y2) - archHeight)
      const cx = (x1 + x2) / 2

      // Quad curve: M x1 y1 Q cx cy x2 y2
      const d = `M ${x1} ${y1} Q ${cx} ${cy} ${x2} ${y2}`

      const isVerb = chunk.pos.includes('verb')
      let stroke = '#818cf8' // Indigo default
      if (isVerb) {
        stroke = '#f87171' // Red-400 for verbs
      } else if (chunk.pos.includes('particle')) {
        stroke = '#60a5fa' // Blue-400 for particles
      } else if (chunk.pos.includes('noun')) {
        stroke = '#4ade80' // Green-400 for nouns
      }

      newArcs.push({ d, stroke, isVerb })
    })

    this.arcs.set(newArcs)
  }

  getChunkStyle(chunk: BunsetsuChunk): string {
    if (chunk.pos.includes('verb')) {
      return 'bg-red-950/60 border-red-500/40 text-red-300'
    } else if (chunk.pos.includes('particle')) {
      return 'bg-blue-950/60 border-blue-500/40 text-blue-300'
    } else if (chunk.pos.includes('noun')) {
      return 'bg-emerald-950/60 border-emerald-500/40 text-emerald-300'
    } else if (chunk.pos.includes('adjective')) {
      return 'bg-amber-950/60 border-amber-500/40 text-amber-300'
    }
    return 'bg-zinc-900 border-zinc-700 text-zinc-200'
  }

  formatPosLabel(pos: string): string {
    return pos
      .replace('noun', 'Substantivo')
      .replace('particle', 'Partícula')
      .replace('verb', 'Verbo')
      .replace('adjective', 'Adjetivo')
      .replace('auxiliary', 'Auxiliar')
      .replace('other', 'Outro')
  }
}
