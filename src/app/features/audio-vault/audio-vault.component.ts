import { Component, OnInit, inject, signal } from '@angular/core'
import { DatePipe } from '@angular/common'
import { GlobalAudioPlayerService } from '../../core/services/global-audio-player.service'
import { TranslocoPipe } from '@ngneat/transloco'
import { AudioTrackMetadata } from '../../core/models/audio.model'

@Component({
  selector: 'app-audio-vault',
  standalone: true,
  imports: [DatePipe, TranslocoPipe],
  templateUrl: './audio-vault.component.html',
})
export class AudioVaultComponent implements OnInit {
  readonly player = inject(GlobalAudioPlayerService)
  readonly importNotice = signal<string | null>(null)
  readonly pendingFolderTracks = signal<AudioTrackMetadata[]>([])

  ngOnInit(): void {
    void this.player.loadHistory()
    void this.player.loadLibrary()
  }

  async importFolder(): Promise<void> {
    this.importNotice.set(null)
    this.pendingFolderTracks.set([])
    try {
      const tracks = await this.player.chooseFolderAndImport()
      if (tracks === null) return
      if (!tracks.length) {
        this.importNotice.set('A pasta selecionada não contém ficheiros de áudio suportados.')
        return
      }
      this.pendingFolderTracks.set(tracks)
      this.importNotice.set(`${tracks.length} faixas importadas para a biblioteca.`)
    } catch (error) {
      this.importNotice.set(`Não foi possível importar a pasta: ${String(error)}`)
    }
  }

  addImportedFolderToQueue(): void {
    this.player.enqueueTracks(this.pendingFolderTracks())
    this.pendingFolderTracks.set([])
    this.importNotice.set('Faixas adicionadas à fila de reprodução.')
  }
}
