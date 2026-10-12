export type SearchSourceType = 'card' | 'note' | 'news'

export interface SearchResult {
  id: string
  sourceType: SearchSourceType
  title: string
  snippet: string
}

export interface SearchSnippetPart {
  text: string
  highlighted: boolean
}
