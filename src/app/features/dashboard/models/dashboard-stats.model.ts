export interface DashboardStats {
  due_today: number
  new_cards: number
  total_cards: number
  total_dictionary_words: number
  total_kanjis: number
  total_tags: number
}

export interface HeatmapEntry {
  date: string
  count: number
}

export interface StreakStats {
  current_streak: number
  max_streak: number
}

