import { describe, it, expect, vi, beforeEach } from 'vitest'
import { KanaTrainerComponent } from './kana-trainer.component'
import { KanaService } from '../../services/kana.service'
import { signal, EnvironmentInjector, createEnvironmentInjector, runInInjectionContext } from '@angular/core'
import { KanaCharacter, KanaMasteryRecord, KanaProficiencyStats } from '../../models/kana.model'

describe('KanaTrainerComponent', () => {
  let component: KanaTrainerComponent
  let kanaServiceMock: {
    proficiency: ReturnType<typeof signal<KanaProficiencyStats>>
    masteryList: ReturnType<typeof signal<KanaMasteryRecord[]>>
    allCharacters: ReturnType<typeof signal<KanaCharacter[]>>
    loadProficiency: ReturnType<typeof vi.fn>
    loadMasteryList: ReturnType<typeof vi.fn>
    loadKanaCharacters: ReturnType<typeof vi.fn>
    updateKanaScore: ReturnType<typeof vi.fn>
    getFilteredCharacters: ReturnType<typeof vi.fn>
  }

  const sampleCharacters: KanaCharacter[] = [
    { character: 'あ', romaji: 'a', alphabet: 'hiragana', category: 'base' },
    { character: 'い', romaji: 'i', alphabet: 'hiragana', category: 'base' },
    { character: 'が', romaji: 'ga', alphabet: 'hiragana', category: 'dakuon' },
  ]

  beforeEach(() => {
    vi.restoreAllMocks()

    kanaServiceMock = {
      proficiency: signal({
        hiragana_proficiency: 50.0,
        katakana_proficiency: 30.0,
        overall_proficiency: 40.0,
        total_attempts: 10,
        total_correct: 8,
      }),
      masteryList: signal([]),
      allCharacters: signal(sampleCharacters),
      loadProficiency: vi.fn().mockResolvedValue({}),
      loadMasteryList: vi.fn().mockResolvedValue([]),
      loadKanaCharacters: vi.fn().mockResolvedValue(sampleCharacters),
      updateKanaScore: vi.fn().mockResolvedValue({}),
      getFilteredCharacters: vi.fn().mockReturnValue(sampleCharacters),
    }

    const providers = [{ provide: KanaService, useValue: kanaServiceMock }]
    const parentInjector = {
      get: (token: unknown) => providers.find((p) => p.provide === token)?.useValue ?? null,
    } as unknown as EnvironmentInjector
    const injector = createEnvironmentInjector([], parentInjector)

    runInInjectionContext(injector, () => {
      component = new KanaTrainerComponent()
    })
  })

  it('should initialize and load kana data on init', async () => {
    await component.ngOnInit()

    expect(kanaServiceMock.loadProficiency).toHaveBeenCalledTimes(1)
    expect(kanaServiceMock.loadMasteryList).toHaveBeenCalledTimes(1)
    expect(kanaServiceMock.loadKanaCharacters).toHaveBeenCalledTimes(1)
    expect(component.currentStep()).toBe('config')
  })

  it('should toggle configuration and start quiz', () => {
    component.setAlphabet('hiragana')
    expect(component.quizConfig().alphabet).toBe('hiragana')

    component.toggleVariation('yoon')
    expect(component.quizConfig().includeYoon).toBe(true)

    component.startQuiz()
    expect(component.currentStep()).toBe('quiz')
    expect(component.currentKana()).not.toBeNull()
  })

  it('should validate correct romaji answer and update score', async () => {
    component.currentKana.set({ character: 'あ', romaji: 'a', alphabet: 'hiragana', category: 'base' })
    component.userInput.set('a')

    await component.checkAnswer()

    expect(component.lastResult()).toBe('correct')
    expect(component.streak()).toBe(1)
    expect(component.sessionCorrect()).toBe(1)
    expect(kanaServiceMock.updateKanaScore).toHaveBeenCalledWith('あ', true)
  })

  it('should flag wrong romaji answer and reset streak', async () => {
    component.currentKana.set({ character: 'あ', romaji: 'a', alphabet: 'hiragana', category: 'base' })
    component.streak.set(5)
    component.userInput.set('o')

    await component.checkAnswer()

    expect(component.lastResult()).toBe('wrong')
    expect(component.streak()).toBe(0)
    expect(component.lastWrongAnswer()?.expected).toBe('a')
    expect(kanaServiceMock.updateKanaScore).toHaveBeenCalledWith('あ', false)
  })

  it('should allow returning back to configuration screen', () => {
    component.startQuiz()
    expect(component.currentStep()).toBe('quiz')

    component.backToConfig()
    expect(component.currentStep()).toBe('config')
  })
})
