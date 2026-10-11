export type OnboardingStep = 'welcome' | 'ai-model' | 'kanjivg' | 'completed'

export interface OnboardingState {
  currentStep: OnboardingStep
  isAiModelDownloaded: boolean
  isKanjiVgImported: boolean
  importedKanjiCount: number
}
