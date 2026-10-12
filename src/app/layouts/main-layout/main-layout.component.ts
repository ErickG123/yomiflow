import { Component, HostListener, OnDestroy, inject } from '@angular/core'
import { Router, RouterOutlet } from '@angular/router'
import { listen } from '@tauri-apps/api/event'
import { TrayAction } from '../../core/models/tray-action.model'
import { ZenModeService } from '../../core/services/zen-mode.service'
import { NotebookService } from '../../features/notebook/services/notebook.service'
import { PomodoroControlService } from '../../features/pomodoro/services/pomodoro-control.service'
import { SidebarComponent } from '../components/sidebar/sidebar.component'
import { TopbarComponent } from '../components/topbar/topbar.component'
import { SettingsModalComponent } from '../components/settings-modal/settings-modal.component'
import { ProcessingOverlayComponent } from '../components/processing-overlay/processing-overlay.component'
import { CommandPaletteComponent } from '../../features/command-palette/command-palette.component'
import { CommandPaletteService } from '../../core/services/command-palette.service'
import { DictionaryTooltipComponent } from '../../features/dictionary/components/dictionary-tooltip/dictionary-tooltip.component'
import { GlobalPlayerComponent } from '../../shared/components/global-player/global-player.component'

@Component({
  selector: 'app-main-layout',
  standalone: true,
  imports: [
    RouterOutlet,
    SidebarComponent,
    TopbarComponent,
    SettingsModalComponent,
    ProcessingOverlayComponent,
    CommandPaletteComponent,
    DictionaryTooltipComponent,
    GlobalPlayerComponent,
  ],
  templateUrl: './main-layout.component.html',
  styleUrl: './main-layout.component.scss',
})
export class MainLayoutComponent implements OnDestroy {
  private readonly router = inject(Router)
  private readonly zenMode = inject(ZenModeService)
  private readonly notebook = inject(NotebookService)
  private readonly pomodoro = inject(PomodoroControlService)
  private readonly commandPalette = inject(CommandPaletteService)
  private unlistenTrayAction?: () => void

  constructor() {
    void listen<TrayAction>('tray-action', ({ payload }) => this.handleTrayAction(payload))
      .then((unlisten) => (this.unlistenTrayAction = unlisten))
      .catch((error) => console.warn('Failed to register system tray listener:', error))
  }

  ngOnDestroy(): void {
    this.unlistenTrayAction?.()
  }

  @HostListener('window:keydown.control.k', ['$event'])
  onCommandPaletteShortcut(event: Event): void {
    event.preventDefault()
    this.commandPalette.toggle()
  }

  private handleTrayAction(action: TrayAction): void {
    switch (action) {
      case 'start_pomodoro':
        this.pomodoro.requestStart()
        break
      case 'new_note':
        this.notebook.requestNewNote()
        void this.router.navigate(['/notebook'])
        break
      case 'pending_reviews':
        void this.router.navigate(['/study'])
        break
      case 'zen_mode':
        void this.zenMode.toggleZenMode()
        break
    }
  }
}
