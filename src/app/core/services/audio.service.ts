import { Injectable } from '@angular/core'

@Injectable({
  providedIn: 'root',
})
export class AudioService {
  private japaneseVoice: SpeechSynthesisVoice | null = null

  constructor() {
    this.initVoices()
  }

  private initVoices(): void {
    if (typeof window === 'undefined' || !('speechSynthesis' in window)) {
      return
    }

    const updateVoices = () => {
      const voices = window.speechSynthesis.getVoices()
      // Look for a voice with ja-JP or Japanese name
      this.japaneseVoice =
        voices.find(
          (v) =>
            v.lang.toLowerCase() === 'ja-jp' ||
            v.lang.toLowerCase() === 'ja' ||
            v.lang.toLowerCase().startsWith('ja_')
        ) || null
    }

    updateVoices()
    if (window.speechSynthesis.onvoiceschanged !== undefined) {
      window.speechSynthesis.onvoiceschanged = updateVoices
    }
  }

  speak(text: string, rate: number = 0.9): void {
    if (typeof window === 'undefined' || !('speechSynthesis' in window)) {
      console.warn('SpeechSynthesis is not supported in this environment.')
      return
    }

    if (!text || text.trim().length === 0) {
      return
    }

    // Cancel any ongoing utterance to avoid queuing delay
    window.speechSynthesis.cancel()

    const utterance = new SpeechSynthesisUtterance(text.trim())
    utterance.lang = 'ja-JP'
    utterance.rate = rate

    if (!this.japaneseVoice) {
      // Try resolving one more time in case voices were loaded asynchronously
      const voices = window.speechSynthesis.getVoices()
      this.japaneseVoice =
        voices.find(
          (v) =>
            v.lang.toLowerCase() === 'ja-jp' ||
            v.lang.toLowerCase() === 'ja' ||
            v.lang.toLowerCase().startsWith('ja_')
        ) || null
    }

    if (this.japaneseVoice) {
      utterance.voice = this.japaneseVoice
    }

    window.speechSynthesis.speak(utterance)
  }
}
