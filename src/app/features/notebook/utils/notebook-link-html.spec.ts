import { describe, expect, it } from 'vitest'
import {
  notebookContentToEditorHtml,
  notebookContentToPlainText,
  notebookTextToEditorHtml,
} from './notebook-link-html'

describe('notebookTextToEditorHtml', () => {
  it('restores multiple wiki links as TipTap nodes', () => {
    expect(notebookTextToEditorHtml('See [[First]] and [[Second]]')).toBe(
      '<p>See <span data-type="bidirectional-link" data-label="First">[[First]]</span> and <span data-type="bidirectional-link" data-label="Second">[[Second]]</span></p>',
    )
  })

  it('escapes note content before generating link HTML', () => {
    expect(notebookTextToEditorHtml('<img src=x> [[<script>]]')).toContain(
      '&lt;img src=x&gt; <span data-type="bidirectional-link" data-label="&lt;script&gt;">[[&lt;script&gt;]]</span>',
    )
  })

  it('preserves formatted HTML when loading a saved note', () => {
    const html =
      '<h2><strong>Heading</strong></h2><table><tbody><tr><td>Cell</td></tr></tbody></table>'
    expect(notebookContentToEditorHtml(html)).toBe(html)
    expect(notebookContentToPlainText(html)).toBe('HeadingCell')
  })

  it('converts legacy plain text notes into editor paragraphs', () => {
    expect(notebookContentToEditorHtml('First line\nSecond line')).toBe(
      '<p>First line</p><p>Second line</p>',
    )
    expect(notebookContentToPlainText('First line\nSecond line')).toBe('First line\nSecond line')
  })
})
