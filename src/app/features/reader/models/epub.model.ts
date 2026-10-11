export interface EpubBookInstance {
  loaded?: { metadata?: Promise<{ title?: string }> }
  renderTo?: (el: HTMLElement, opts: unknown) => unknown
  destroy?: () => void
}

export interface EpubRenditionInstance {
  hooks?: { content?: { register?: (cb: (contents: { document: Document }) => Promise<void>) => void } }
  display?: (target?: string) => Promise<void> | void
  next?: () => void
  prev?: () => void
  on?: (event: string, callback: (...args: unknown[]) => void) => void
  destroy?: () => void
}
