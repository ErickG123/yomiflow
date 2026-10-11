import { ChangeDetectionStrategy, Component, computed, inject, input } from '@angular/core'
import { TranslocoPipe } from '@ngneat/transloco'
import { AudioService } from '../../../core/services/audio.service'
import { PitchMora } from './models/pitch-accent.model'

@Component({
  selector: 'app-pitch-accent',
  standalone: true,
  imports: [TranslocoPipe],
  templateUrl: './pitch-accent.component.html',
  styleUrl: './pitch-accent.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class PitchAccentComponent {
  private readonly audioService = inject(AudioService)

  readonly reading = input.required<string>()
  readonly pitch = input<string | number[] | null | undefined>(null)
  readonly showAudioButton = input<boolean>(true)

  readonly moras = computed<PitchMora[]>(() => {
    const rawReading = this.reading() || ''
    if (!rawReading) {
      return []
    }

    // Decompose reading into Japanese moras (handling small kana like ゃ, ゅ, ょ, ぁ, etc.)
    const smallKanas = new Set(['ゃ', 'ゅ', 'ょ', 'ぁ', 'ぃ', 'ぅ', 'ぇ', 'ぉ', 'ゎ', 'ャ', 'ュ', 'ョ', 'ァ', 'ィ', 'ゥ', 'ェ', 'ォ', 'ヮ', 'ッ', 'っ'])
    const chars = Array.from(rawReading)
    const tokens: string[] = []

    for (let i = 0; i < chars.length; i++) {
      const c = chars[i]
      if (i > 0 && smallKanas.has(c)) {
        tokens[tokens.length - 1] += c
      } else {
        tokens.push(c)
      }
    }

    const pitchPattern = this.pitch()
    const result: PitchMora[] = []

    // 1. Array format like [0, 1, 1, 0]
    if (Array.isArray(pitchPattern)) {
      for (let i = 0; i < tokens.length; i++) {
        const isHigh = pitchPattern[i] === 1
        const nextIsLow = i + 1 < pitchPattern.length ? pitchPattern[i + 1] === 0 : false
        const isDrop = isHigh && nextIsLow
        result.push({
          kana: tokens[i],
          isHigh,
          isDrop,
        })
      }
      return result
    }

    // 2. String format like "0,1,1,0", "0110", "HL", "LHL", or numeric accent drop index like "0", "1", "2"
    if (typeof pitchPattern === 'string') {
      const trimmed = pitchPattern.trim()

      if (trimmed.includes(',') || /^[01]+$/.test(trimmed)) {
        const digits = trimmed.includes(',')
          ? trimmed.split(',').map((s) => s.trim())
          : trimmed.split('')

        for (let i = 0; i < tokens.length; i++) {
          const isHigh = digits[i] === '1' || digits[i] === 'H' || digits[i] === 'h'
          const nextIsLow = i + 1 < digits.length ? (digits[i + 1] === '0' || digits[i + 1] === 'L' || digits[i + 1] === 'l') : false
          result.push({
            kana: tokens[i],
            isHigh,
            isDrop: isHigh && nextIsLow,
          })
        }
        return result
      }

      if (/^[HLhl]+$/.test(trimmed)) {
        const letters = trimmed.toUpperCase().split('')
        for (let i = 0; i < tokens.length; i++) {
          const isHigh = letters[i] === 'H'
          const nextIsLow = i + 1 < letters.length ? letters[i + 1] === 'L' : false
          result.push({
            kana: tokens[i],
            isHigh,
            isDrop: isHigh && nextIsLow,
          })
        }
        return result
      }

      // If it's a number indicating the downstep mora position (0 = heiban, 1 = atamadaka, 2 = nakadaka, etc.)
      const dropIndex = parseInt(trimmed, 10)
      if (!isNaN(dropIndex)) {
        for (let i = 0; i < tokens.length; i++) {
          const moraNumber = i + 1
          const isHigh =
            dropIndex === 0
              ? moraNumber > 1
              : dropIndex === 1
                ? moraNumber === 1
                : moraNumber > 1 && moraNumber <= dropIndex
          const isDrop = isHigh && moraNumber === dropIndex
          result.push({
            kana: tokens[i],
            isHigh,
            isDrop,
          })
        }
        return result
      }
    }

    // Default fallback when pitch is absent or unrecognized: flat display
    return tokens.map((k) => ({
      kana: k,
      isHigh: false,
      isDrop: false,
    }))
  })

  speak(): void {
    const text = this.reading()
    if (text) {
      this.audioService.speak(text)
    }
  }
}
