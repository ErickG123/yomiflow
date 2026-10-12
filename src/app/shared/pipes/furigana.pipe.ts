import { Pipe, PipeTransform, SecurityContext, inject } from '@angular/core'
import { DomSanitizer } from '@angular/platform-browser'
import { invoke } from '@tauri-apps/api/core'

@Pipe({
  name: 'furigana',
  standalone: true,
})
export class FuriganaPipe implements PipeTransform {
  private readonly sanitizer = inject(DomSanitizer)

  async transform(text: string | null | undefined): Promise<string> {
    if (!text) return ''

    const generatedHtml = await invoke<string>('generate_furigana', { text })
    return this.sanitizer.sanitize(SecurityContext.HTML, generatedHtml) ?? ''
  }
}
