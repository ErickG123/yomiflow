export interface MorphologicalToken {
  surface: string
  reading?: string | null
  pos: 'verb' | 'noun' | 'particle' | 'adjective' | 'auxiliary' | 'other' | string
}

export interface NotebookEntry {
  id: string
  title: string
  content_md: string
  created_at: string
  updated_at: string
}

export type AnalysisMode = 'auto' | 'manual'

