import { InputRule, Node, mergeAttributes } from '@tiptap/core'
import { BidirectionalLinkOptions } from '../models/bidirectional-link.model'

export const BidirectionalLinkExtension = Node.create<BidirectionalLinkOptions>({
  name: 'bidirectionalLink',
  inline: true,
  group: 'inline',
  atom: true,
  selectable: false,
  addOptions() {
    return { onOpenLink: () => undefined }
  },
  addAttributes() {
    return {
      label: {
        default: '',
        parseHTML: (element) => element.getAttribute('data-label') ?? '',
        renderHTML: (attributes) => ({ 'data-label': attributes['label'] }),
      },
    }
  },
  parseHTML() {
    return [{ tag: 'span[data-type="bidirectional-link"]' }]
  },
  renderHTML({ node, HTMLAttributes }) {
    const label = String(node.attrs['label'] ?? '')
    return [
      'span',
      mergeAttributes(HTMLAttributes, {
        'data-type': 'bidirectional-link',
        class:
          'text-blue-400 underline decoration-blue-500/50 underline-offset-2 cursor-pointer hover:text-blue-300',
      }),
      `[[${label}]]`,
    ]
  },
  renderText({ node }) {
    return `[[${String(node.attrs['label'] ?? '')}]]`
  },
  addInputRules() {
    return [
      new InputRule({
        find: /\[\[([^\]]+)\]\]$/,
        handler: ({ state, range, match }) => {
          const label = match[1]?.trim()
          if (label) state.tr.replaceWith(range.from, range.to, this.type.create({ label }))
        },
      }),
    ]
  },
  addNodeView() {
    return ({ node }) => {
      let currentNode = node
      const dom = document.createElement('span')
      dom.contentEditable = 'false'
      dom.setAttribute('data-type', 'bidirectional-link')
      dom.setAttribute('role', 'link')
      dom.setAttribute('tabindex', '0')
      dom.className =
        'text-blue-400 underline decoration-blue-500/50 underline-offset-2 cursor-pointer hover:text-blue-300'

      const updateLabel = (): void => {
        const label = String(currentNode.attrs['label'] ?? '')
        dom.textContent = `[[${label}]]`
        dom.setAttribute('aria-label', `Abrir nota ${label}`)
      }
      const openLink = (event: Event): void => {
        event.preventDefault()
        event.stopPropagation()
        this.options.onOpenLink(String(currentNode.attrs['label'] ?? ''))
      }
      const openLinkOnKeyDown = (event: KeyboardEvent): void => {
        if (event.key === 'Enter' || event.key === ' ') openLink(event)
      }
      dom.addEventListener('click', openLink)
      dom.addEventListener('keydown', openLinkOnKeyDown)
      updateLabel()

      return {
        dom,
        update: (updatedNode) => {
          if (updatedNode.type !== this.type) return false
          currentNode = updatedNode
          updateLabel()
          return true
        },
        stopEvent: (event) => event.type === 'click' || event.type === 'keydown',
        ignoreMutation: () => true,
        destroy: () => {
          dom.removeEventListener('click', openLink)
          dom.removeEventListener('keydown', openLinkOnKeyDown)
        },
      }
    }
  },
})
