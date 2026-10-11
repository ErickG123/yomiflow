/**
 * Pure domain utilities for string sanitation and Japanese text formatting
 * adheres strictly to DRY & functional architecture.
 */

/**
 * Strips punctuation and symbols from OCR/subtitle text while preserving
 * Japanese Kanji, Hiragana, Katakana, and Japanese punctuation.
 */
export function sanitizeJapaneseText(raw: string | null | undefined): string {
  if (!raw) return ''
  return raw.replace(/[|\\_~`^'"\-=+*<>(){}[\]/]/g, '').trim()
}

/**
 * Tokenizes a sentence into interactive words/tokens.
 * Supports whitespace separation or individual glyph segments.
 */
export function tokenizeJapaneseSentence(text: string | null | undefined): string[] {
  if (!text) return []
  const trimmed = text.trim()
  if (trimmed.includes(' ')) {
    return trimmed.split(' ')
  }
  return trimmed.split('')
}
