import { Component, inject } from '@angular/core'
import { AppStateService } from '../../../core/services/app-state.service'

@Component({
  selector: 'app-processing-overlay',
  standalone: true,
  templateUrl: './processing-overlay.component.html',
})
export class ProcessingOverlayComponent {
  readonly appState = inject(AppStateService)
}
