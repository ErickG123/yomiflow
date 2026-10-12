import { describe, expect, it } from 'vitest'
import { extractJapaneseWord } from './japanese-word-extraction'

describe('extractJapaneseWord', () => {
  it('extracts a contiguous Japanese word under the caret', () => {
    expect(extractJapaneseWord('今日は学生です。', 4)).toBe('学生')
  })

  it('supports hiragana, katakana, and supplementary kanji', () => {
    expect(extractJapaneseWord('かなカナ𠮷野家', 5)).toBe('𠮷')
  })

  it('returns null outside Japanese text and for invalid offsets', () => {
    expect(extractJapaneseWord('hello 日本語', 2)).toBeNull()
    expect(extractJapaneseWord('日本語', -1)).toBeNull()
    expect(extractJapaneseWord('日本語', 4)).toBeNull()
  })

  it('can resolve a caret at the end of a Japanese text node', () => {
    expect(extractJapaneseWord('勉強する', 4)).toBe('勉強する')
  })
})
