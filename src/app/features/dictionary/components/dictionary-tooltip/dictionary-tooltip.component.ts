import { ChangeDetectionStrategy, Component, computed, inject } from '@angular/core'
import { ContextualDictionaryService } from '../../services/contextual-dictionary.service'

@Component({
  selector: 'app-dictionary-tooltip',
  standalone: true,
  templateUrl: './dictionary-tooltip.component.html',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class DictionaryTooltipComponent {
  readonly dictionary = inject(ContextualDictionaryService)
  readonly left = computed(() =>
    Math.max(8, Math.min(this.dictionary.position().x + 14, window.innerWidth - 300)),
  )
  readonly top = computed(() =>
    Math.max(8, Math.min(this.dictionary.position().y + 18, window.innerHeight - 150)),
  )
}
