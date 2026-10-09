import { DecimalPipe } from '@angular/common'
import { ChangeDetectionStrategy, Component, OnInit, computed, inject } from '@angular/core'
import { Router } from '@angular/router'
import { TranslocoPipe } from '@ngneat/transloco'
import { DashboardService } from '../../services/dashboard.service'

export interface HeatmapDay {
  date: string
  count: number
}

@Component({
  selector: 'app-dashboard-main',
  standalone: true,
  imports: [TranslocoPipe, DecimalPipe],
  templateUrl: './dashboard-main.component.html',
  styleUrl: './dashboard-main.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DashboardMainComponent implements OnInit {
  private readonly dashboardService = inject(DashboardService)
  private readonly router = inject(Router)

  readonly stats = this.dashboardService.stats
  readonly heatmapData = this.dashboardService.heatmapData
  readonly streakStats = this.dashboardService.streakStats
  readonly isLoading = this.dashboardService.isLoading

  // Generates the past 16 weeks (112 days) of heatmap squares
  readonly calendarDays = computed<HeatmapDay[]>(() => {
    const rawData = this.heatmapData()
    const map = new Map<string, number>()
    for (const item of rawData) {
      map.set(item.date, item.count)
    }

    const days: HeatmapDay[] = []
    const totalDays = 112 // 16 weeks of 7 days
    const today = new Date()

    // End at today or alignment with end of week
    for (let i = totalDays - 1; i >= 0; i--) {
      const d = new Date(today)
      d.setDate(today.getDate() - i)
      const year = d.getFullYear()
      const month = String(d.getMonth() + 1).padStart(2, '0')
      const day = String(d.getDate()).padStart(2, '0')
      const dateStr = `${year}-${month}-${day}`
      days.push({
        date: dateStr,
        count: map.get(dateStr) ?? 0,
      })
    }
    return days
  })

  ngOnInit(): void {
    this.dashboardService.getStats()
    this.dashboardService.getHeatmap()
    this.dashboardService.getStreaks()
  }

  startStudy(): void {
    const { due_today, new_cards } = this.stats()
    if (due_today > 0 || new_cards > 0) {
      this.router.navigate(['/study'])
    }
  }
}

