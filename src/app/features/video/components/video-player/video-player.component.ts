import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  OnInit,
  ViewChild,
  inject,
  signal,
} from '@angular/core'
import { TranslocoPipe } from '@ngneat/transloco'
import { VideoService } from '../../services/video.service'
import { DictionaryService } from '../../../dictionary/services/dictionary.service'
import { DictionaryEntry } from '../../../dictionary/models/dictionary.model'
import { AudioService } from '../../../../core/services/audio.service'
import { tokenizeJapaneseSentence } from '../../../../shared/utils/text-utils'
import { PitchAccentComponent } from '../../../../shared/components/pitch-accent/pitch-accent.component'

@Component({
  selector: 'app-video-player',
  standalone: true,
  imports: [TranslocoPipe, PitchAccentComponent],
  templateUrl: './video-player.component.html',
  styleUrl: './video-player.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class VideoPlayerComponent implements OnInit {
  readonly videoService = inject(VideoService)
  readonly dictionaryService = inject(DictionaryService)
  readonly audioService = inject(AudioService)

  @ViewChild('videoElement') videoRef?: ElementRef<HTMLVideoElement>

  ngOnInit(): void {
    void this.videoService.checkFfmpeg()
  }

  readonly isDrawerOpen = signal<boolean>(false)
  readonly selectedWord = signal<string | null>(null)
  readonly dictionaryResults = this.dictionaryService.searchResults
  readonly kanjiResults = this.dictionaryService.kanjiSearchResults
  readonly isSearching = this.dictionaryService.isLoading

  async openVideo(): Promise<void> {
    await this.videoService.openVideoDialog()
  }

  onTimeUpdate(): void {
    const video = this.videoRef?.nativeElement
    if (video) {
      this.videoService.updateCurrentTime(video.currentTime)
    }
  }

  // Hover Dictionary: Pause video and search word
  async onWordHover(word: string): Promise<void> {
    const video = this.videoRef?.nativeElement
    if (video && !video.paused) {
      video.pause()
    }

    const cleaned = word.replace(/[.,/#!$%^&*;:{}=\-_`~()?"'「」『』]/g, '').trim()
    if (!cleaned) {
      return
    }

    this.selectedWord.set(cleaned)
    this.isDrawerOpen.set(true)
    await this.dictionaryService.search(cleaned)
  }

  closeDrawer(): void {
    this.isDrawerOpen.set(false)
  }

  readonly isCreatingCardWithAudio = signal<boolean>(false)

  async toggleSrs(entry: DictionaryEntry): Promise<void> {
    if (entry.in_srs) {
      await this.dictionaryService.removeFromSrs(entry.id)
    } else {
      await this.dictionaryService.addToSrs(entry.id)
    }
  }

  async createFlashcardWithAudio(entry: DictionaryEntry): Promise<void> {
    const dialogue = this.videoService.currentDialogue()
    if (!dialogue) {
      alert('Nenhuma legenda ativa no momento para recortar áudio.')
      return
    }

    this.isCreatingCardWithAudio.set(true)
    try {
      // 1. Cut native audio clip with FFmpeg
      const filename = `clip_${Date.now()}_${dialogue.id || 1}`
      const audioPath = await this.videoService.cutAudioClip(
        dialogue.start_time,
        dialogue.end_time,
        filename
      )

      // 2. Add entry to SRS
      await this.dictionaryService.addToSrs(entry.id)

      alert(`Flashcard criado com sucesso com áudio nativo!\nÁudio salvo em: ${audioPath}`)
    } catch (err) {
      console.error('Failed to create flashcard with audio:', err)
      alert(`Erro ao criar flashcard com áudio nativo: ${err}`)
    } finally {
      this.isCreatingCardWithAudio.set(false)
    }
  }

  // Split subtitle sentence into individual interactive tokens/words
  tokenize(text: string): string[] {
    return tokenizeJapaneseSentence(text)
  }
}
