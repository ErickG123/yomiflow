import { describe, it, expect, vi, beforeEach } from 'vitest'
import { ShadowingTrainerComponent } from './shadowing-trainer.component'
import { ShadowingComparisonResult, ShadowingService } from '../../services/shadowing.service'
import { AudioService } from '../../../../core/services/audio.service'
import { signal, EnvironmentInjector, createEnvironmentInjector, runInInjectionContext } from '@angular/core'

describe('ShadowingTrainerComponent', () => {
  let component: ShadowingTrainerComponent

  let mockShadowingService: {
    isRecording: ReturnType<typeof signal<boolean>>
    transcribedText: ReturnType<typeof signal<string>>
    isEvaluating: ReturnType<typeof signal<boolean>>
    comparisonResult: ReturnType<typeof signal<ShadowingComparisonResult | null>>
    speechError: ReturnType<typeof signal<string | null>>
    startRecording: ReturnType<typeof vi.fn>
    stopRecordingAndEvaluate: ReturnType<typeof vi.fn>
  }

  let mockAudioService: {
    speak: ReturnType<typeof vi.fn>
  }

  beforeEach(() => {
    vi.restoreAllMocks()

    mockShadowingService = {
      isRecording: signal(false),
      transcribedText: signal(''),
      isEvaluating: signal(false),
      comparisonResult: signal(null),
      speechError: signal(null),
      startRecording: vi.fn(),
      stopRecordingAndEvaluate: vi.fn().mockReturnValue({
        expectedText: 'テスト',
        transcribedText: 'テスト',
        accuracy: 100,
        matchedWords: [],
      }),
    }

    mockAudioService = {
      speak: vi.fn(),
    }

    const providers = [
      { provide: ShadowingService, useValue: mockShadowingService },
      { provide: AudioService, useValue: mockAudioService },
    ]

    const parentInjector = {
      get: (token: unknown) => {
        const found = providers.find((p) => p.provide === token)
        if (found) return found.useValue
        return null
      },
    } as unknown as EnvironmentInjector

    const injector = createEnvironmentInjector(providers, parentInjector)

    runInInjectionContext(injector, () => {
      component = new ShadowingTrainerComponent()
    })
  })

  it('should create ShadowingTrainerComponent', () => {
    expect(component).toBeTruthy()
  })

  it('should call tts service on speakTts', () => {
    component.targetText.set('おはよう')
    component.speakTts()
    expect(mockAudioService.speak).toHaveBeenCalledWith('おはよう')
  })

  it('should start recording if not recording', () => {
    component.toggleRecording()
    expect(mockShadowingService.startRecording).toHaveBeenCalled()
  })

  it('should stop and evaluate if already recording', () => {
    mockShadowingService.isRecording.set(true)
    component.toggleRecording()
    expect(mockShadowingService.stopRecordingAndEvaluate).toHaveBeenCalled()
  })
})
