import { Component, inject } from '@angular/core'
import { FormsModule } from '@angular/forms'
import { RouterLink } from '@angular/router'
import { GlobalAudioPlayerService } from '../../../core/services/global-audio-player.service'
import { LayoutStateService } from '../../../layouts/services/layout-state.service'
import { ZenModeService } from '../../../core/services/zen-mode.service'

@Component({
  selector: 'app-global-player',
  standalone: true,
  imports: [FormsModule, RouterLink],
  templateUrl: './global-player.component.html',
  styleUrl: './global-player.component.scss',
})
export class GlobalPlayerComponent {
  readonly player = inject(GlobalAudioPlayerService)
  readonly layoutState = inject(LayoutStateService)
  readonly zenMode = inject(ZenModeService)

  sidebarOffset(): number {
    return this.layoutState.isSidebarCollapsed() || this.zenMode.isZenMode() ? 80 : 256
  }
}
