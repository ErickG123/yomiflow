import { Injectable, signal } from '@angular/core'
import { SettingsTab } from '../../core/models/settings.model'

@Injectable({ providedIn: 'root' })
export class LayoutStateService {
  readonly isSidebarCollapsed = signal(false)
  readonly isSidebarOpen = signal(false)
  readonly isSettingsOpen = signal(false)
  readonly settingsActiveTab = signal<SettingsTab>('general')

  toggleSettings(): void {
    this.isSettingsOpen.update((open) => !open)
  }
}
