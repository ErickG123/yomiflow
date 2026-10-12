import { describe, expect, it, vi } from 'vitest'
import { FuriganaPipe } from './furigana.pipe'
import { DomSanitizer } from '@angular/platform-browser'
import { SecurityContext } from '@angular/core'
import { createEnvironmentInjector, runInInjectionContext } from '@angular/core'
import * as tauriCore from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

describe('FuriganaPipe', () => {
  it('sanitizes generated ruby HTML before returning it to innerHTML', async () => {
    vi.mocked(tauriCore.invoke).mockResolvedValue(
      '<ruby>漢字<rt>かんじ</rt></ruby><script>alert(1)</script>',
    )
    const sanitizer = { sanitize: vi.fn(() => '<ruby>漢字<rt>かんじ</rt></ruby>') }
    const injector = createEnvironmentInjector([{ provide: DomSanitizer, useValue: sanitizer }])
    const pipe = runInInjectionContext(injector, () => new FuriganaPipe())

    const result = await pipe.transform('漢字')

    expect(result).toBe('<ruby>漢字<rt>かんじ</rt></ruby>')
    expect(result).not.toContain('<script>')
    expect(sanitizer.sanitize).toHaveBeenCalledWith(
      SecurityContext.HTML,
      expect.stringContaining('<script>'),
    )
  })
})
