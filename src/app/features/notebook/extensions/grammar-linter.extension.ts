import { Editor, Extension } from '@tiptap/core'
import { Plugin, PluginKey } from '@tiptap/pm/state'
import { Decoration, DecorationSet, EditorView } from '@tiptap/pm/view'
import { LintDecoration, LintSuggestion } from '../models/notebook.model'

type GrammarLinterState = { decorations: DecorationSet }

type HoverHandler = (lint: LintDecoration | null, coordinates: { x: number; y: number }) => void

interface GrammarLinterOptions {
  onHover: HoverHandler | null
}

export const grammarLinterPluginKey = new PluginKey<GrammarLinterState>('grammarLinter')

export const getGrammarLintText = (editor: Editor): string => {
  const blocks: string[] = []
  editor.state.doc.descendants((node) => {
    if (node.isTextblock) blocks.push(node.textContent)
  })
  return blocks.join('\n')
}

export const setGrammarLintDecorations = (editor: Editor, suggestions: LintSuggestion[]): void => {
  const blocks: Array<{ text: string; offset: number; position: number }> = []
  let offset = 0
  editor.state.doc.descendants((node, position) => {
    if (!node.isTextblock) return
    if (blocks.length) offset += 1
    blocks.push({ text: node.textContent, offset, position: position + 1 })
    offset += node.textContent.length
  })

  const decorations: Decoration[] = []
  suggestions.forEach((suggestion, lintIndex) => {
    const block = blocks.find(
      (candidate) =>
        suggestion.index_start >= candidate.offset &&
        suggestion.index_end <= candidate.offset + candidate.text.length,
    )
    if (!block) return

    const from = block.position + suggestion.index_start - block.offset
    const to = block.position + suggestion.index_end - block.offset
    if (from < to) {
      decorations.push(
        Decoration.inline(
          from,
          to,
          {
            class: 'grammar-linter-suggestion',
            'data-grammar-lint-index': String(lintIndex),
            style:
              'text-decoration: underline wavy #f97316; text-underline-offset: 3px; cursor: help;',
          },
          { lintIndex, suggestion, from, to },
        ),
      )
    }
  })

  editor.view.dispatch(
    editor.state.tr.setMeta(grammarLinterPluginKey, {
      decorations: DecorationSet.create(editor.state.doc, decorations),
    } satisfies GrammarLinterState),
  )
}

export const GrammarLinterExtension = Extension.create<GrammarLinterOptions>({
  name: 'grammarLinter',

  addOptions() {
    return { onHover: null }
  },

  addProseMirrorPlugins() {
    const onHover = this.options.onHover
    return [
      new Plugin<GrammarLinterState>({
        key: grammarLinterPluginKey,
        state: {
          init: () => ({ decorations: DecorationSet.empty }),
          apply(transaction, previous) {
            const next = transaction.getMeta(grammarLinterPluginKey) as
              GrammarLinterState | undefined
            if (next) return next
            if (transaction.docChanged) return { decorations: DecorationSet.empty }
            return previous
          },
        },
        props: {
          decorations(state) {
            return grammarLinterPluginKey.getState(state)?.decorations ?? DecorationSet.empty
          },
          handleDOMEvents: {
            mouseover(view, event) {
              return reportHoveredLint(view, event, onHover)
            },
            click(view, event) {
              return reportHoveredLint(view, event, onHover)
            },
            mouseout(_view, event) {
              const mouseEvent = event as MouseEvent
              const related = mouseEvent.relatedTarget
              if (related instanceof Element && related.closest('.grammar-linter-suggestion')) {
                return false
              }
              onHover?.(null, { x: mouseEvent.clientX, y: mouseEvent.clientY })
              return false
            },
          },
        },
      }),
    ]
  },
})

function reportHoveredLint(view: EditorView, event: Event, onHover: HoverHandler | null): false {
  const target = event.target
  if (!(target instanceof Element)) return false
  const lintElement = target.closest<HTMLElement>('.grammar-linter-suggestion')
  const index = Number(lintElement?.dataset['grammarLintIndex'])
  if (lintElement && Number.isInteger(index)) {
    const decoration = grammarLinterPluginKey
      .getState(view.state)
      ?.decorations.find()
      .find((candidate) => candidate.spec['lintIndex'] === index)
    if (decoration) {
      const spec = decoration.spec as { suggestion: LintSuggestion }
      onHover?.(
        { suggestion: spec.suggestion, from: decoration.from, to: decoration.to },
        { x: (event as MouseEvent).clientX, y: (event as MouseEvent).clientY },
      )
    }
  }
  return false
}
