import { Editor } from '@tiptap/core'
import StarterKit from '@tiptap/starter-kit'
import { afterEach, describe, expect, it, vi } from 'vitest'
import { BidirectionalLinkExtension } from './bidirectional-link.extension'

describe('BidirectionalLinkExtension', () => {
  let editor: Editor | undefined

  afterEach(() => {
    editor?.destroy()
    editor = undefined
  })

  it('turns multiple [[note]] inputs on one line into clickable inline nodes', () => {
    const openLink = vi.fn()
    editor = new Editor({
      element: document.createElement('div'),
      extensions: [StarterKit, BidirectionalLinkExtension.configure({ onOpenLink: openLink })],
      content: '<p></p>',
    })

    for (const character of '[[First Note]] [[Second Note]]') {
      const { from, to } = editor.state.selection
      const handled = editor.view.someProp('handleTextInput', (handler) =>
        handler(editor!.view, from, to, character),
      )
      if (!handled) editor.commands.insertContentAt({ from, to }, character)
    }

    const links = editor.view.dom.querySelectorAll('[data-type="bidirectional-link"]')
    expect(links).toHaveLength(2)
    expect(links[0].textContent).toBe('[[First Note]]')
    expect(links[1].textContent).toBe('[[Second Note]]')
    expect(editor.getText()).toBe('[[First Note]] [[Second Note]]')

    links[0].dispatchEvent(new MouseEvent('click', { bubbles: true }))
    expect(openLink).toHaveBeenCalledWith('First Note')
  })
})
