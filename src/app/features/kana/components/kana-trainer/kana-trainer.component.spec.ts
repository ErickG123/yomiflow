import { describe, it, expect, vi, beforeEach } from 'vitest'
import { KanaTrainerComponent } from './kana-trainer.component'
import { KanaService } from '../../services/kana.service'
import { signal, EnvironmentInjector, createEnvironmentInjector, runInInjectionContext } from '@angular/core'
import { KanaMasteryRecord, KanaProficiencyStats } from '../../models/kana.model'

describe('KanaTrainerComponent', () => {
  let component: KanaTrainerComponent
  let kanaServiceMock: {
    proficiency: ReturnType<typeof signal<KanaProficiencyStats>>
    masteryList: ReturnType<typeof signal<KanaMasteryRecord[]>>
    loadProficiency: ReturnType<typeof vi.fn>
    loadMasteryList: ReturnType<typeof vi.fn>
    recordAttempt: ReturnType<typeof vi.fn>
    getCharacters: ReturnType<typeof vi.fn>
  }

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
      loadProficiency: vi.fn().mockResolvedValue({}),
      loadMasteryList: vi.fn().mockResolvedValue([]),
      recordAttempt: vi.fn().mockResolvedValue({}),
      getCharacters: vi.fn().mockReturnValue([
        { character: 'あ', romaji: 'a', type: 'hiragana', row: 'a' },
        { character: 'い', romaji: 'i', type: 'hiragana', row: 'a' },
      ]),
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

  it('should initialize and load a random kana question', async () => {
    await component.ngOnInit()

    expect(kanaServiceMock.loadProficiency).toHaveBeenCalledTimes(1)
    expect(kanaServiceMock.loadMasteryList).toHaveBeenCalledTimes(1)
    expect(component.currentKana()).not.toBeNull()
  })

  it('should validate correct romaji answer and update streak', async () => {
    component.currentKana.set({ character: 'あ', romaji: 'a', type: 'hiragana', row: 'a' })
    component.userInput.set('a')

    await component.checkAnswer()

    expect(component.lastResult()).toBe('correct')
    expect(component.streak()).toBe(1)
    expect(component.sessionCorrect()).toBe(1)
    expect(kanaServiceMock.recordAttempt).toHaveBeenCalledWith('あ', 'a', 'hiragana', true)
  })

  it('should flag wrong romaji answer and reset streak', async () => {
    component.currentKana.set({ character: 'あ', romaji: 'a', type: 'hiragana', row: 'a' })
    component.streak.set(5)
    component.userInput.set('o')

    await component.checkAnswer()

    expect(component.lastResult()).toBe('wrong')
    expect(component.streak()).toBe(0)
    expect(component.lastWrongAnswer()?.expected).toBe('a')
    expect(kanaServiceMock.recordAttempt).toHaveBeenCalledWith('あ', 'a', 'hiragana', false)
  })

  it('should compute accuracy percentage accurately', () => {
    component.sessionTotal.set(10)
    component.sessionCorrect.set(8)
    expect(component.getAccuracyPercentage()).toBe(80)
  })
})
