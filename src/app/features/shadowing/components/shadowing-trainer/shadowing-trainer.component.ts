import {
  ChangeDetectionStrategy,
  Component,
  ElementRef,
  OnDestroy,
  ViewChild,
  inject,
  signal,
} from '@angular/core'
import { CommonModule } from '@angular/common'
import { FormsModule } from '@angular/forms'
import { TranslocoDirective, TranslocoPipe } from '@ngneat/transloco'
import { ShadowingService } from '../../services/shadowing.service'
import { AudioService } from '../../../../core/services/audio.service'

@Component({
  selector: 'app-shadowing-trainer',
  standalone: true,
  imports: [CommonModule, FormsModule, TranslocoDirective],
  templateUrl: './shadowing-trainer.component.html',
  styleUrl: './shadowing-trainer.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class ShadowingTrainerComponent implements OnDestroy {
  readonly shadowingService = inject(ShadowingService)
  readonly audioService = inject(AudioService)

  @ViewChild('audioPlayer') audioPlayerRef?: ElementRef<HTMLAudioElement>

  readonly targetText = signal<string>('日本の文化を勉強しています。')
  readonly audioSrc = signal<string | null>(null)
  readonly audioFileName = signal<string | null>(null)
  readonly isPlayingAudio = signal<boolean>(false)

  onAudioFileSelected(event: Event): void {
    const input = event.target as HTMLInputElement
    if (input.files && input.files.length > 0) {
      const file = input.files[0]
      this.audioFileName.set(file.name)
      const url = URL.createObjectURL(file)
      this.audioSrc.set(url)
    }
  }

  playAudio(): void {
    const audio = this.audioPlayerRef?.nativeElement
    if (audio) {
      void audio.play()
      this.isPlayingAudio.set(true)
    }
  }

  pauseAudio(): void {
    const audio = this.audioPlayerRef?.nativeElement
    if (audio) {
      audio.pause()
      this.isPlayingAudio.set(false)
    }
  }

  speakTts(): void {
    const text = this.targetText().trim()
    if (text) {
      this.audioService.speak(text)
    }
  }

  toggleRecording(): void {
    if (this.shadowingService.isRecording()) {
      this.shadowingService.stopRecordingAndEvaluate(this.targetText())
    } else {
      this.shadowingService.startRecording()
    }
  }

  ngOnDestroy(): void {
    if (this.audioSrc()) {
      URL.revokeObjectURL(this.audioSrc()!)
    }
  }
}
