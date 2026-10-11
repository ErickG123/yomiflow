import { ChangeDetectionStrategy, Component, OnInit, input, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { KanjiComponent, KanjiDetails } from './models/kanji-breakdown.model'

@Component({
  selector: 'app-kanji-breakdown',
  standalone: true,
  templateUrl: './kanji-breakdown.component.html',
  styleUrl: './kanji-breakdown.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class KanjiBreakdownComponent implements OnInit {
  readonly kanji = input.required<string>()

  readonly components = signal<KanjiComponent[]>([])
  readonly mnemonicText = signal<string>('')
  readonly isLoading = signal<boolean>(false)
  readonly isSaved = signal<boolean>(false)

  async ngOnInit(): Promise<void> {
    await this.loadDetails()
  }

  async loadDetails(): Promise<void> {
    const character = this.kanji()
    if (!character) return

    this.isLoading.set(true)
    try {
      const details = await invoke<KanjiDetails>('get_kanji_details', {
        kanji: character,
      })
      this.components.set(details.components ?? [])
      this.mnemonicText.set(details.mnemonic ?? '')
    } catch (err) {
      console.warn('Erro ao carregar detalhes do kanji:', err)
    } finally {
      this.isLoading.set(false)
    }
  }

  onTextInput(event: Event): void {
    const target = event.target as HTMLTextAreaElement
    this.mnemonicText.set(target.value)
    this.isSaved.set(false)
  }

  async onBlurSave(): Promise<void> {
    const character = this.kanji()
    const text = this.mnemonicText()
    if (!character) return

    try {
      await invoke('save_kanji_mnemonic', {
        kanji: character,
        mnemonic: text,
      })
      this.isSaved.set(true)
      setTimeout(() => {
        this.isSaved.set(false)
      }, 2500)
    } catch (err) {
      console.error('Erro ao guardar mnemónica:', err)
    }
  }
}
