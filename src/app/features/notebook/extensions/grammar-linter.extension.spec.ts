import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import { afterEach, describe, expect, it } from 'vitest'
import {
  GrammarLinterExtension,
  grammarLinterPluginKey,
  setGrammarLintDecorations,
} from './grammar-linter.extension'

describe('GrammarLinterExtension', () => {
  let editor: Editor | null = null

  afterEach(() => {
    editor?.destroy()
    editor = null
  })

  it('decorates the UTF-16 range and clears the decoration after replacing the particle', () => {
    const element = document.createElement('div')
    editor = new Editor({
      element,
      extensions: [StarterKit, GrammarLinterExtension],
      content: '<p>日本語を分かります</p>',
    })

    setGrammarLintDecorations(editor, [
      {
        index_start: 3,
        index_end: 4,
        original: 'を',
        suggestion: 'が',
        reason: 'Potential verbs generally use が',
      },
    ])

    const decoration = grammarLinterPluginKey.getState(editor.state)?.decorations.find()[0]
    expect(decoration?.from).toBe(4)
    expect(decoration?.to).toBe(5)
    expect(editor.view.dom.querySelector('.grammar-linter-suggestion')?.textContent).toBe('を')

    editor.commands.insertContentAt({ from: 4, to: 5 }, 'が')

    expect(editor.getText()).toBe('日本語が分かります')
    expect(grammarLinterPluginKey.getState(editor.state)?.decorations.find()).toHaveLength(0)
  })

  it('maps lint offsets across multiple text blocks', () => {
    const element = document.createElement('div')
    editor = new Editor({
      element,
      extensions: [StarterKit, GrammarLinterExtension],
      content: '<p>日本語を分かります</p><p>日本語を分かります</p>',
    })

    setGrammarLintDecorations(editor, [
      {
        index_start: 13,
        index_end: 14,
        original: 'を',
        suggestion: 'が',
        reason: 'Potential verbs generally use が',
      },
    ])

    const decoration = grammarLinterPluginKey.getState(editor.state)?.decorations.find()[0]
    expect(decoration?.from).toBe(15)
    expect(editor.view.dom.querySelectorAll('.grammar-linter-suggestion')).toHaveLength(1)
  })
})
