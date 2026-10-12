import {
  createEnvironmentInjector,
  EnvironmentInjector,
  runInInjectionContext,
} from '@angular/core'
import { DomSanitizer } from '@angular/platform-browser'
import { SafeHtmlPipe } from './safe-html.pipe'

describe('SafeHtmlPipe', () => {
  it('removes executable markup while preserving safe formatting', () => {
    const sanitizer = {
      sanitize: vi.fn((_context, value: string) =>
        value.replace(/<script.*?<\/script>/gi, '').replace(/ onerror="[^"]*"/gi, ''),
      ),
    }
    const parent = { get: () => null } as unknown as EnvironmentInjector
    const injector = createEnvironmentInjector(
      [{ provide: DomSanitizer, useValue: sanitizer }],
      parent,
    )
    const pipe = runInInjectionContext(injector, () => new SafeHtmlPipe())

    const result = pipe.transform(
      '<p>safe</p><img src=x onerror="alert(1)"><script>alert(2)</script>',
    )

    expect(result).toContain('<p>safe</p>')
    expect(result).not.toMatch(/onerror|<script/i)
  })

  it('returns an empty string for missing content', () => {
    const parent = { get: () => null } as unknown as EnvironmentInjector
    const injector = createEnvironmentInjector(
      [{ provide: DomSanitizer, useValue: { sanitize: vi.fn() } }],
      parent,
    )
    const pipe = runInInjectionContext(injector, () => new SafeHtmlPipe())
    expect(pipe.transform(null)).toBe('')
  })
})
