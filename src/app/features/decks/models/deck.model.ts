export interface Deck {
  id: string
  name: string
  description?: string | null
}

export interface DeckWithCounts {
  id: string
  name: string
  description?: string | null
  new_count: number
  due_count: number
  total_count: number
}

export interface Tag {
  id: string
  name: string
  color?: string | null
}

export interface CreateDeckPayload {
  name: string
  description?: string | null
}

export interface UpdateDeckPayload {
  id: string
  name: string
  description?: string | null
}
