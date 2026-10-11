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
  deck_id?: string | null
  next_review_date?: number | null
  srs_stage?: number | null
}

export type AnalysisMode = 'auto' | 'manual'

export interface SlashMenuItem {
  id: string
  titleKey: string
  descKey: string
  icon: string
  action: () => void
}

