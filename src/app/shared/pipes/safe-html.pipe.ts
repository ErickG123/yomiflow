import { Pipe, PipeTransform, SecurityContext, inject } from '@angular/core'
import { DomSanitizer } from '@angular/platform-browser'

@Pipe({
  name: 'safeHtml',
  standalone: true,
})
export class SafeHtmlPipe implements PipeTransform {
  private readonly sanitizer = inject(DomSanitizer)

  transform(value: string | null | undefined): string {
    if (!value) {
      return ''
    }
    // Sanitize against malicious XSS payloads (e.g. script injection in SVGs or rich text)
    const sanitized = this.sanitizer.sanitize(SecurityContext.HTML, value) ?? ''
    return sanitized
  }
}
