const JAPANESE_CHARACTER = /^[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}ー々〆ヵヶ]$/u

export function extractJapaneseWord(text: string, offset: number): string | null {
  if (!Number.isInteger(offset) || offset < 0 || offset > text.length || text.length === 0) {
    return null
  }

  const characters: Array<{ value: string; start: number; end: number }> = []
  let codeUnitOffset = 0
  for (const value of text) {
    const start = codeUnitOffset
    codeUnitOffset += value.length
    characters.push({ value, start, end: codeUnitOffset })
  }

  let index = characters.findIndex(({ start, end }) => offset >= start && offset < end)
  if (index < 0 && offset === text.length) index = characters.length - 1
  if (index < 0 || !JAPANESE_CHARACTER.test(characters[index].value)) return null

  if (typeof Intl.Segmenter === 'function') {
    const segments = new Intl.Segmenter('ja', { granularity: 'word' }).segment(text)
    for (const segment of segments) {
      if (
        segment.isWordLike &&
        offset >= segment.index &&
        offset < segment.index + segment.segment.length &&
        [...segment.segment].some((character) => JAPANESE_CHARACTER.test(character))
      ) {
        return segment.segment
      }
    }
  }

  let first = index
  let last = index
  while (first > 0 && JAPANESE_CHARACTER.test(characters[first - 1].value)) first--
  while (last + 1 < characters.length && JAPANESE_CHARACTER.test(characters[last + 1].value)) last++
  return text.slice(characters[first].start, characters[last].end)
}
