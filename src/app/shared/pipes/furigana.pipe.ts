import { Pipe, PipeTransform, inject } from '@angular/core'
import { DomSanitizer, SafeHtml } from '@angular/platform-browser'
import { invoke } from '@tauri-apps/api/core'

@Pipe({
  name: 'furigana',
  standalone: true,
})
export class FuriganaPipe implements PipeTransform {
  private readonly sanitizer = inject(DomSanitizer)

  async transform(text: string | null | undefined): Promise<SafeHtml> {
    if (!text) return this.sanitizer.bypassSecurityTrustHtml('')

    const generatedHtml = await invoke<string>('generate_furigana', { text })
    return this.sanitizer.bypassSecurityTrustHtml(generatedHtml)
  }
}
