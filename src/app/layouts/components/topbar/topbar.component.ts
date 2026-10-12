import { Component, inject } from '@angular/core'
import { TranslocoDirective } from '@ngneat/transloco'
import { ZenModeService } from '../../../core/services/zen-mode.service'

@Component({
  selector: 'app-topbar',
  standalone: true,
  imports: [TranslocoDirective],
  templateUrl: './topbar.component.html',
})
export class TopbarComponent {
  readonly zenModeService = inject(ZenModeService)
}
