export interface ShadowingComparisonResult {
  expectedText: string
  transcribedText: string
  accuracy: number
  matchedWords: { word: string; isCorrect: boolean }[]
}

export interface SpeechRecognitionInterface {
  lang: string
  continuous: boolean
  interimResults: boolean
  start: () => void
  stop: () => void
  onresult: ((event: { results: ArrayLike<{ [index: number]: { transcript: string } }> }) => void) | null
  onerror: ((event: { error: string }) => void) | null
  onend: (() => void) | null
}

export interface SpeechRecognitionConstructor {
  new (): SpeechRecognitionInterface
}

export interface WindowWithSpeech {
  SpeechRecognition?: SpeechRecognitionConstructor
  webkitSpeechRecognition?: SpeechRecognitionConstructor
}
