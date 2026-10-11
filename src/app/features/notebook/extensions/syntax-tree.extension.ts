import { Node, mergeAttributes } from '@tiptap/core'
import { ApplicationRef, EnvironmentInjector, createComponent } from '@angular/core'
import { SyntaxNodeComponent } from '../components/syntax-node/syntax-node.component'

export interface SyntaxTreeOptions {
  injector: EnvironmentInjector
  appRef: ApplicationRef
}

export const SyntaxTreeExtension = Node.create<SyntaxTreeOptions>({
  name: 'syntaxTree',
  group: 'block',
  atom: true,
  draggable: true,

  addAttributes() {
    return {
      chunks: {
        default: '[]',
        parseHTML: (element) => element.getAttribute('data-chunks') || '[]',
        renderHTML: (attributes) => ({
          'data-chunks': typeof attributes['chunks'] === 'string'
            ? attributes['chunks']
            : JSON.stringify(attributes['chunks']),
        }),
      },
    }
  },

  parseHTML() {
    return [
      {
        tag: 'div[data-type="syntax-tree"]',
      },
    ]
  },

  renderHTML({ HTMLAttributes }) {
    return ['div', mergeAttributes({ 'data-type': 'syntax-tree' }, HTMLAttributes)]
  },

  addNodeView() {
    return ({ node }) => {
      const dom = document.createElement('div')
      dom.setAttribute('data-type', 'syntax-tree')
      dom.className = 'syntax-tree-node-wrapper select-none'

      const injector = this.options.injector
      const appRef = this.options.appRef

      let componentRef: ReturnType<typeof createComponent<SyntaxNodeComponent>> | null = null

      if (injector && appRef) {
        componentRef = createComponent(SyntaxNodeComponent, {
          environmentInjector: injector,
          hostElement: dom,
        })

        const rawChunks = node.attrs['chunks']
        componentRef.instance.data = rawChunks
        componentRef.changeDetectorRef.detectChanges()
        appRef.attachView(componentRef.hostView)
      }

      return {
        dom,
        update: (updatedNode) => {
          if (updatedNode.type.name !== this.name) {
            return false
          }
          if (componentRef) {
            componentRef.instance.data = updatedNode.attrs['chunks']
            componentRef.changeDetectorRef.detectChanges()
          }
          return true
        },
        destroy: () => {
          if (componentRef) {
            appRef?.detachView(componentRef.hostView)
            componentRef.destroy()
            componentRef = null
          }
        },
      }
    }
  },
})
