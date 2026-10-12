import { Injectable, signal } from '@angular/core'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import {
  AudioPlayerSnapshot,
  AudioTrackMetadata,
  PassiveListeningEntry,
} from '../models/audio.model'

@Injectable({ providedIn: 'root' })
export class GlobalAudioPlayerService {
  readonly track = signal<AudioTrackMetadata | null>(null)
  readonly isPlaying = signal(false)
  readonly positionMs = signal(0)
  readonly volume = signal(0.75)
  readonly queue = signal<AudioTrackMetadata[]>([])
  readonly history = signal<PassiveListeningEntry[]>([])
  readonly library = signal<AudioTrackMetadata[]>([])
  private pollId?: ReturnType<typeof setInterval>

  async chooseAndPlay(): Promise<void> {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: 'Audio', extensions: ['mp3', 'wav', 'flac', 'ogg', 'm4a'] }],
    })
    const path = typeof selected === 'string' ? selected : selected?.[0]
    if (path) await this.playPath(path)
  }

  async playPath(path: string): Promise<void> {
    const track = await invoke<AudioTrackMetadata>('play_audio', { path })
    this.track.set(track)
    this.positionMs.set(0)
    this.isPlaying.set(true)
    this.queue.update((queue) =>
      queue.some((item) => item.path === path) ? queue : [...queue, track],
    )
    this.library.update((library) => [track, ...library.filter((item) => item.path !== track.path)])
    this.startPolling()
  }

  async togglePlayback(): Promise<void> {
    if (!this.track()) return this.chooseAndPlay()
    if (this.isPlaying()) {
      await invoke('pause_audio')
      this.isPlaying.set(false)
    } else {
      await invoke('resume_audio')
      this.isPlaying.set(true)
      this.startPolling()
    }
  }

  async setVolume(value: number): Promise<void> {
    const volume = Math.max(0, Math.min(1, value))
    this.volume.set(volume)
    await invoke('set_volume', { volume })
  }

  async seek(value: number): Promise<void> {
    const position = Math.max(0, Math.min(this.track()?.duration_ms ?? 0, value))
    this.positionMs.set(position)
    await invoke('seek_to', { positionMs: position })
  }

  async playNext(direction: -1 | 1): Promise<void> {
    const queue = this.queue()
    if (!queue.length) return
    const currentIndex = queue.findIndex((item) => item.path === this.track()?.path)
    const nextIndex = (currentIndex + direction + queue.length) % queue.length
    await this.playPath(queue[nextIndex].path)
  }

  async loadHistory(): Promise<void> {
    this.history.set(await invoke<PassiveListeningEntry[]>('get_passive_listening_history'))
  }

  async loadLibrary(): Promise<void> {
    this.library.set(await invoke<AudioTrackMetadata[]>('get_audio_library'))
  }

  async chooseFolderAndImport(): Promise<AudioTrackMetadata[] | null> {
    const selected = await open({ multiple: false, directory: true })
    if (typeof selected !== 'string') return null
    const tracks = await invoke<AudioTrackMetadata[]>('import_audio_folder', {
      folderPath: selected,
    })
    if (tracks.length) {
      const importedPaths = new Set(tracks.map((track) => track.path))
      this.library.update((library) => [
        ...tracks,
        ...library.filter((track) => !importedPaths.has(track.path)),
      ])
    }
    return tracks
  }

  enqueueTracks(tracks: AudioTrackMetadata[]): void {
    this.queue.update((queue) => {
      const queuedPaths = new Set(queue.map((track) => track.path))
      return [...queue, ...tracks.filter((track) => !queuedPaths.has(track.path))]
    })
  }

  formatTime(milliseconds: number): string {
    const totalSeconds = Math.floor(milliseconds / 1000)
    return `${Math.floor(totalSeconds / 60)}:${String(totalSeconds % 60).padStart(2, '0')}`
  }

  private startPolling(): void {
    if (this.pollId) clearInterval(this.pollId)
    this.pollId = setInterval(() => void this.refresh(), 700)
  }

  private async refresh(): Promise<void> {
    try {
      const snapshot = await invoke<AudioPlayerSnapshot>('get_audio_player_snapshot')
      this.track.set(snapshot.track)
      this.positionMs.set(snapshot.position_ms)
      this.isPlaying.set(snapshot.is_playing)
      this.volume.set(snapshot.volume)
      if (!snapshot.is_playing) {
        clearInterval(this.pollId)
        this.pollId = undefined
      }
    } catch (error) {
      console.warn('Unable to refresh global audio player state', error)
    }
  }
}
