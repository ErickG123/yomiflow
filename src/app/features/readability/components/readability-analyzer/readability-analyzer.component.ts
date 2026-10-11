import { Component, computed, signal } from '@angular/core'
import { DecimalPipe } from '@angular/common'
import { FormsModule } from '@angular/forms'
import { TranslocoPipe } from '@ngneat/transloco'
import { invoke } from '@tauri-apps/api/core'
import { ReadabilityAnalysis, ReadabilityTextSegment } from '../../models/readability.model'

@Component({
  selector: 'app-readability-analyzer',
  standalone: true,
  imports: [DecimalPipe, FormsModule, TranslocoPipe],
  templateUrl: './readability-analyzer.component.html',
  styleUrl: './readability-analyzer.component.scss',
})
export class ReadabilityAnalyzerComponent {
  readonly text = signal('')
  readonly analysis = signal<ReadabilityAnalysis | null>(null)
  readonly isAnalyzing = signal(false)
  readonly errorMessage = signal<string | null>(null)
  readonly segments = computed<ReadabilityTextSegment[]>(() => {
    const analysis = this.analysis()
    const text = this.text()
    if (!analysis) return [{ text, isUnknown: false }]

    const result: ReadabilityTextSegment[] = []
    let cursor = 0
    for (const word of analysis.unknownWords) {
      const start = Math.max(cursor, Math.min(text.length, word.start))
      const end = Math.max(start, Math.min(text.length, word.end))
      if (start > cursor) result.push({ text: text.slice(cursor, start), isUnknown: false })
      if (end > start) result.push({ text: text.slice(start, end), isUnknown: true })
      cursor = end
    }
    if (cursor < text.length) result.push({ text: text.slice(cursor), isUnknown: false })
    return result
  })
  readonly ringDasharray = computed(() => {
    const percent = this.analysis()?.comprehensionPercentage ?? 0
    const circumference = 2 * Math.PI * 42
    return `${(percent / 100) * circumference} ${circumference}`
  })

  async analyze(): Promise<void> {
    if (!this.text().trim() || this.isAnalyzing()) return
    this.isAnalyzing.set(true)
    this.errorMessage.set(null)
    this.analysis.set(null)
    try {
      this.analysis.set(await invoke<ReadabilityAnalysis>('analyze_text_readability', { text: this.text() }))
    } catch (error) {
      this.errorMessage.set(String(error))
    } finally {
      this.isAnalyzing.set(false)
    }
  }

  updateText(value: string): void {
    this.text.set(value)
    this.analysis.set(null)
    this.errorMessage.set(null)
  }
}
