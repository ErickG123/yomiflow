import { Component, inject } from '@angular/core'
import { RouterLink, RouterLinkActive } from '@angular/router'
import { TranslocoPipe } from '@ngneat/transloco'
import { PomodoroWidgetComponent } from '../../../features/pomodoro/components/pomodoro-widget/pomodoro-widget.component'
import { ZenModeService } from '../../../core/services/zen-mode.service'
import { LayoutStateService } from '../../services/layout-state.service'

@Component({
  selector: 'app-sidebar',
  standalone: true,
  imports: [RouterLink, RouterLinkActive, TranslocoPipe, PomodoroWidgetComponent],
  templateUrl: './sidebar.component.html',
})
export class SidebarComponent {
  readonly layoutState = inject(LayoutStateService)
  readonly zenModeService = inject(ZenModeService)

  effectiveSidebarCollapsed(): boolean {
    return this.zenModeService.isZenMode() || this.layoutState.isSidebarCollapsed()
  }

  toggleSidebar(): void {
    this.layoutState.isSidebarCollapsed.update((collapsed) => !collapsed)
  }

  toggleZenMode(): void {
    this.zenModeService.toggleZenMode()
  }

  toggleSettings(): void {
    this.layoutState.toggleSettings()
  }
}
