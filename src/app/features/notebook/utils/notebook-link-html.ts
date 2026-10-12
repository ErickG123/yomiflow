function escapeHtml(value: string): string {
  return value
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;')
}

export function notebookTextToEditorHtml(text: string): string {
  return text
    .split(/\r?\n/)
    .map((line) => {
      const escapedLine = escapeHtml(line)
      const withLinks = escapedLine.replace(/\[\[([^\]]+)\]\]/g, (_match, rawLabel: string) => {
        const label = rawLabel.trim()
        return `<span data-type="bidirectional-link" data-label="${label}">[[${label}]]</span>`
      })
      return `<p>${withLinks || '<br>'}</p>`
    })
    .join('')
}

const HTML_CONTENT_PATTERN =
  /<(?:p|h[1-6]|ul|ol|li|table|thead|tbody|tr|th|td|blockquote|div|span|br|strong|em|s|code)\b/i

export function notebookContentToEditorHtml(content: string): string {
  return HTML_CONTENT_PATTERN.test(content) ? content : notebookTextToEditorHtml(content)
}

export function notebookContentToPlainText(content: string): string {
  if (!HTML_CONTENT_PATTERN.test(content)) return content

  const parsed = new DOMParser().parseFromString(content, 'text/html')
  parsed.querySelectorAll('script, style').forEach((node) => node.remove())
  return parsed.body.textContent ?? ''
}
