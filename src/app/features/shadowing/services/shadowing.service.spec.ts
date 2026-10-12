import { describe, it, expect } from 'vitest'
import { ShadowingService } from './shadowing.service'

describe('ShadowingService', () => {
  let service: ShadowingService

  beforeEach(() => {
    service = new ShadowingService()
  })

  it('should initialize with default states', () => {
    expect(service.isRecording()).toBe(false)
    expect(service.transcribedText()).toBe('')
    expect(service.comparisonResult()).toBeNull()
  })

  it('should calculate 100% accuracy for identical text', () => {
    const res = service.calculateAccuracy('日本語の勉強', '日本語の勉強')
    expect(res.accuracy).toBe(100)
    expect(res.matchedWords.every((w) => w.isCorrect)).toBe(true)
  })

  it('should calculate partial accuracy for slight pronunciation variations', () => {
    const res = service.calculateAccuracy('こんにちは', 'こんには')
    expect(res.accuracy).toBeGreaterThan(60)
    expect(res.accuracy).toBeLessThan(100)
  })

  it('should handle punctuation ignoring cleanly', () => {
    const res = service.calculateAccuracy('桜が咲いた！', '桜が咲いた')
    expect(res.accuracy).toBe(100)
  })

  it('shows a friendly message when native Japanese recognition has a network error', () => {
    const recognition = {
      lang: '',
      continuous: false,
      interimResults: false,
      onresult: null as
        ((event: { results: ArrayLike<{ 0: { transcript: string } }> }) => void) | null,
      onerror: null,
      onend: null as (() => void) | null,
      start() {},
      stop() {},
    }
    const original = (window as Window & { SpeechRecognition?: unknown }).SpeechRecognition
    Object.defineProperty(window, 'SpeechRecognition', {
      configurable: true,
      value: class {
        constructor() {
          return recognition
        }
      },
    })
    try {
      service = new ShadowingService()
      service.startRecording()
      recognition.onerror?.({ error: 'network' })

      expect(service.speechError()).toBe(
        'O reconhecimento de voz nativo requer conexão com a internet. Verifique sua rede.',
      )
      expect(service.isRecording()).toBe(false)
    } finally {
      Object.defineProperty(window, 'SpeechRecognition', { configurable: true, value: original })
    }
  })
})
