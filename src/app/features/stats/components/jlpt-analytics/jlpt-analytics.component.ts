import {
  Component,
  OnInit,
  AfterViewInit,
  OnDestroy,
  ElementRef,
  ViewChild,
  inject,
} from '@angular/core'
import { CommonModule } from '@angular/common'
import { TranslocoDirective } from '@ngneat/transloco'
import {
  Chart,
  RadarController,
  RadialLinearScale,
  PointElement,
  LineElement,
  Filler,
  Tooltip,
  Legend,
} from 'chart.js'
import { JlptStatsService } from '../../services/jlpt-stats.service'

Chart.register(
  RadarController,
  RadialLinearScale,
  PointElement,
  LineElement,
  Filler,
  Tooltip,
  Legend
)

@Component({
  selector: 'app-jlpt-analytics',
  standalone: true,
  imports: [CommonModule, TranslocoDirective],
  templateUrl: './jlpt-analytics.component.html',
  styleUrl: './jlpt-analytics.component.scss',
})
export class JlptAnalyticsComponent implements OnInit, AfterViewInit, OnDestroy {
  @ViewChild('radarCanvas') radarCanvas?: ElementRef<HTMLCanvasElement>

  readonly statsService = inject(JlptStatsService)
  private chartInstance: Chart | null = null

  async ngOnInit(): Promise<void> {
    await this.statsService.fetchJlptStats()
    this.renderRadarChart()
  }

  ngAfterViewInit(): void {
    this.renderRadarChart()
  }


  ngOnDestroy(): void {
    if (this.chartInstance) {
      this.chartInstance.destroy()
      this.chartInstance = null
    }
  }

  renderRadarChart(): void {
    if (!this.radarCanvas) return

    const stats = this.statsService.stats()
    if (!stats || !stats.levels || stats.levels.length === 0) return

    if (this.chartInstance) {
      this.chartInstance.destroy()
      this.chartInstance = null
    }

    const ctx = this.radarCanvas.nativeElement.getContext('2d')
    if (!ctx) return

    const labels = stats.levels.map((l) => l.level_name)
    const kanjiData = stats.levels.map((l) => l.kanji_percentage)
    const vocabData = stats.levels.map((l) => l.vocab_percentage)

    this.chartInstance = new Chart(ctx, {
      type: 'radar',
      data: {
        labels,
        datasets: [
          {
            label: 'Kanji (%)',
            data: kanjiData,
            backgroundColor: 'rgba(99, 102, 241, 0.25)',
            borderColor: '#6366f1',
            borderWidth: 2,
            pointBackgroundColor: '#818cf8',
            pointBorderColor: '#fff',
            pointHoverBackgroundColor: '#fff',
            pointHoverBorderColor: '#818cf8',
          },
          {
            label: 'Vocabulário (%)',
            data: vocabData,
            backgroundColor: 'rgba(16, 185, 129, 0.2)',
            borderColor: '#10b981',
            borderWidth: 2,
            pointBackgroundColor: '#34d399',
            pointBorderColor: '#fff',
            pointHoverBackgroundColor: '#fff',
            pointHoverBorderColor: '#34d399',
          },
        ],
      },
      options: {
        responsive: true,
        maintainAspectRatio: false,
        scales: {
          r: {
            angleLines: {
              color: 'rgba(255, 255, 255, 0.1)',
            },
            grid: {
              color: 'rgba(255, 255, 255, 0.1)',
            },
            pointLabels: {
              color: '#d4d4d8',
              font: {
                size: 13,
                weight: 'bold',
              },
            },
            ticks: {
              color: '#a1a1aa',
              backdropColor: 'transparent',
              stepSize: 20,
            },
            suggestedMin: 0,
            suggestedMax: 100,
          },
        },
        plugins: {
          legend: {
            position: 'top',
            labels: {
              color: '#e4e4e7',
              font: {
                size: 12,
              },
            },
          },
          tooltip: {
            backgroundColor: '#18181b',
            titleColor: '#fafafa',
            bodyColor: '#e4e4e7',
            borderColor: '#27272a',
            borderWidth: 1,
            padding: 10,
          },
        },
      },
    })
  }
}
