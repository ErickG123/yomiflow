import { Component, HostBinding, inject, signal } from '@angular/core'
import { FormsModule } from '@angular/forms'
import { RouterLink } from '@angular/router'
import { GlobalAudioPlayerService } from '../../../core/services/global-audio-player.service'
import { LayoutStateService } from '../../../layouts/services/layout-state.service'
import { ZenModeService } from '../../../core/services/zen-mode.service'
import { TranslocoPipe } from '@ngneat/transloco'

@Component({
  selector: 'app-global-player',
  standalone: true,
  imports: [FormsModule, RouterLink, TranslocoPipe],
  templateUrl: './global-player.component.html',
  styleUrl: './global-player.component.scss',
})
export class GlobalPlayerComponent {
  readonly player = inject(GlobalAudioPlayerService)
  readonly layoutState = inject(LayoutStateService)
  readonly zenMode = inject(ZenModeService)
  readonly isPlayerExpanded = signal(true)

  @HostBinding('style.height')
  get reservedHeight(): string {
    return this.isPlayerExpanded() ? '5rem' : '2rem'
  }

  @HostBinding('style.flex-basis')
  get reservedFlexBasis(): string {
    return this.reservedHeight
  }

  togglePlayer(): void {
    this.isPlayerExpanded.update((expanded) => !expanded)
  }

  sidebarOffset(): number {
    return this.layoutState.isSidebarCollapsed() || this.zenMode.isZenMode() ? 80 : 256
  }
}
