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
})
