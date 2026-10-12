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
