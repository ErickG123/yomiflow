import { Injectable, inject, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { convertFileSrc } from '@tauri-apps/api/core'
import { AppStateService } from '../../../core/services/app-state.service'
import { SubtitleDialogue } from '../../../shared/models/media.model'

@Injectable({
  providedIn: 'root',
})
export class VideoService {
  private readonly appState = inject(AppStateService)

  readonly currentVideoPath = signal<string | null>(null)
  readonly currentVideoAssetUrl = signal<string | null>(null)
  readonly subtitles = signal<SubtitleDialogue[]>([])
  readonly currentDialogue = signal<SubtitleDialogue | null>(null)
  readonly isLoading = signal<boolean>(false)
  readonly isFfmpegAvailable = signal<boolean>(true)

  async checkFfmpeg(): Promise<boolean> {
    try {
      const available = await invoke<boolean>('check_ffmpeg_installed')
      this.isFfmpegAvailable.set(available)
      return available
    } catch {
      this.isFfmpegAvailable.set(false)
      return false
    }
  }

  async openVideoDialog(): Promise<boolean> {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [
          {
            name: 'Vídeo (*.mkv, *.mp4, *.webm, *.avi)',
            extensions: ['mkv', 'mp4', 'webm', 'avi'],
          },
        ],
      })

      if (!selected) {
        return false
      }

      const filePath = typeof selected === 'string' ? selected : selected[0]
      if (!filePath) {
        return false
      }

      return await this.loadVideo(filePath)
    } catch (error) {
      console.error('Failed to open video file:', error)
      alert(`Erro ao abrir vídeo: ${error}`)
      return false
    }
  }

  async loadVideo(filePath: string): Promise<boolean> {
    this.isLoading.set(true)
    this.appState.isProcessing.set(true)
    this.appState.progressMessage.set('A carregar vídeo e extrair legendas...')

    try {
      this.currentVideoPath.set(filePath)
      // Convert filesystem path to tauri asset:// url
      const assetUrl = convertFileSrc(filePath)
      this.currentVideoAssetUrl.set(assetUrl)

      // Extract subtitles via backend (FFmpeg)
      try {
        const subs = await invoke<SubtitleDialogue[]>('extract_subtitles', {
          videoPath: filePath,
        })
        this.subtitles.set(subs)
      } catch (subErr) {
        console.warn('Subtitle extraction warning:', subErr)
        this.subtitles.set([])
      }

      return true
    } catch (err) {
      console.error('Error loading video:', err)
      alert(`Erro ao carregar o vídeo: ${err}`)
      return false
    } finally {
      this.isLoading.set(false)
      this.appState.isProcessing.set(false)
    }
  }

  updateCurrentTime(currentTime: number): void {
    const subs = this.subtitles()
    const active = subs.find(
      (s) => currentTime >= s.start_time && currentTime <= s.end_time
    )
    this.currentDialogue.set(active || null)
  }

  async cutAudioClip(
    startTime: number,
    endTime: number,
    outputFilename: string
  ): Promise<string> {
    const videoPath = this.currentVideoPath()
    if (!videoPath) {
      throw new Error('Nenhum vídeo carregado')
    }

    return await invoke<string>('cut_audio_clip', {
      videoPath,
      startTime,
      endTime,
      outputFilename,
    })
  }
}
