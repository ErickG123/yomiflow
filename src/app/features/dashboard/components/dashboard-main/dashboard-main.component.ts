import { DecimalPipe } from '@angular/common'
import { ChangeDetectionStrategy, Component, OnInit, inject } from '@angular/core'
import { Router } from '@angular/router'
import { TranslocoPipe } from '@ngneat/transloco'
import { DashboardService } from '../../services/dashboard.service'

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
  readonly isLoading = this.dashboardService.isLoading

  ngOnInit(): void {
    this.dashboardService.getStats()
  }

  startStudy(): void {
    const { due_today, new_cards } = this.stats()
    if (due_today > 0 || new_cards > 0) {
      this.router.navigate(['/study'])
    }
  }
}
