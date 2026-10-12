import { describe, expect, it } from 'vitest'
import { notebookTextToEditorHtml } from './notebook-link-html'

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
})
