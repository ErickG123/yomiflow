import { Component, OnInit, inject } from '@angular/core'
import { DatePipe } from '@angular/common'
import { GlobalAudioPlayerService } from '../../core/services/global-audio-player.service'

@Component({
  selector: 'app-audio-vault',
  standalone: true,
  imports: [DatePipe],
  templateUrl: './audio-vault.component.html',
})
export class AudioVaultComponent implements OnInit {
  readonly player = inject(GlobalAudioPlayerService)

  ngOnInit(): void {
    void this.player.loadHistory()
    void this.player.loadLibrary()
  }
}
