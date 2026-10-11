import { Injectable, signal } from '@angular/core'

export interface ShadowingComparisonResult {
  expectedText: string
  transcribedText: string
  accuracy: number
  matchedWords: { word: string; isCorrect: boolean }[]
}

interface SpeechRecognitionInterface {
  lang: string
  continuous: boolean
  interimResults: boolean
  start: () => void
  stop: () => void
  onresult: ((event: { results: ArrayLike<{ [index: number]: { transcript: string } }> }) => void) | null
  onerror: ((event: { error: string }) => void) | null
  onend: (() => void) | null
}

interface SpeechRecognitionConstructor {
  new (): SpeechRecognitionInterface
}

interface WindowWithSpeech {
  SpeechRecognition?: SpeechRecognitionConstructor
  webkitSpeechRecognition?: SpeechRecognitionConstructor
}

@Injectable({
  providedIn: 'root',
})
export class ShadowingService {
  readonly isRecording = signal<boolean>(false)
  readonly transcribedText = signal<string>('')
  readonly isEvaluating = signal<boolean>(false)
  readonly comparisonResult = signal<ShadowingComparisonResult | null>(null)
  readonly speechError = signal<string | null>(null)

  private recognition: SpeechRecognitionInterface | null = null

  constructor() {
    this.initSpeechRecognition()
  }

  private initSpeechRecognition(): void {
    const speechWindow = window as unknown as WindowWithSpeech
    const SpeechRecognition = speechWindow.SpeechRecognition || speechWindow.webkitSpeechRecognition
    if (SpeechRecognition) {
      const rec = new SpeechRecognition()
      rec.lang = 'ja-JP'
      rec.continuous = true
      rec.interimResults = false

      rec.onresult = (event) => {
        let text = ''
        for (let i = 0; i < event.results.length; i++) {
          text += event.results[i][0].transcript
        }
        this.transcribedText.set(text.trim())
      }

      rec.onerror = (event) => {
        console.warn('Speech recognition error:', event.error)
        this.speechError.set(`Erro no reconhecimento de voz: ${event.error}`)
        this.isRecording.set(false)
      }

      rec.onend = () => {
        this.isRecording.set(false)
      }

      this.recognition = rec
    }
  }

  startRecording(): void {
    this.speechError.set(null)
    this.transcribedText.set('')
    this.comparisonResult.set(null)

    if (!this.recognition) {
      this.speechError.set('Web Speech API não disponível neste navegador.')
      return
    }

    try {
      this.isRecording.set(true)
      this.recognition.start()
    } catch (err) {
      console.warn('Could not start recognition:', err)
      this.isRecording.set(false)
    }
  }

  stopRecordingAndEvaluate(targetText: string): ShadowingComparisonResult {
    if (this.recognition && this.isRecording()) {
      try {
        this.recognition.stop()
      } catch {
        // ignore
      }
    }
    this.isRecording.set(false)

    const transcribed = this.transcribedText()
    const result = this.calculateAccuracy(targetText, transcribed)
    this.comparisonResult.set(result)
    return result
  }

  calculateAccuracy(expected: string, transcribed: string): ShadowingComparisonResult {
    const cleanExpected = expected.replace(/[^\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}ーa-zA-Z0-9]/gu, '')
    const cleanTranscribed = transcribed.replace(/[^\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}ーa-zA-Z0-9]/gu, '')

    if (!cleanExpected) {
      return {
        expectedText: expected,
        transcribedText: transcribed,
        accuracy: 0,
        matchedWords: [],
      }
    }

    // Levenshtein distance on characters
    const dist = this.levenshtein(cleanExpected, cleanTranscribed)
    const maxLen = Math.max(cleanExpected.length, cleanTranscribed.length)
    const accuracy = maxLen === 0 ? 100 : Math.round(Math.max(0, (1 - dist / maxLen)) * 100)

    // Segment expected into individual characters/words for visual feedback
    const matchedWords = Array.from(cleanExpected).map((char) => ({
      word: char,
      isCorrect: cleanTranscribed.includes(char),
    }))

    return {
      expectedText: expected,
      transcribedText: transcribed,
      accuracy,
      matchedWords,
    }
  }

  private levenshtein(a: string, b: string): number {
    const matrix: number[][] = []
    for (let i = 0; i <= b.length; i++) {
      matrix[i] = [i]
    }
    for (let j = 0; j <= a.length; j++) {
      matrix[0][j] = j
    }
    for (let i = 1; i <= b.length; i++) {
      for (let j = 1; j <= a.length; j++) {
        if (b.charAt(i - 1) === a.charAt(j - 1)) {
          matrix[i][j] = matrix[i - 1][j - 1]
        } else {
          matrix[i][j] = Math.min(
            matrix[i - 1][j - 1] + 1,
            matrix[i][j - 1] + 1,
            matrix[i - 1][j] + 1
          )
        }
      }
    }
    return matrix[b.length][a.length]
  }
}
