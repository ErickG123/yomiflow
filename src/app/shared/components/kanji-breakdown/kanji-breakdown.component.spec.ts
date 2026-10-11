import { describe, it, expect, vi, beforeEach } from 'vitest'
import { KanjiBreakdownComponent } from './kanji-breakdown.component'
import * as tauriCore from '@tauri-apps/api/core'
import { EnvironmentInjector, createEnvironmentInjector, runInInjectionContext } from '@angular/core'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('KanjiBreakdownComponent', () => {
  let component: KanjiBreakdownComponent

  beforeEach(() => {
    vi.restoreAllMocks()

    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_kanji_details') {
        return {
          kanji: '森',
          components: [{ kanji: '森', component: '木', meaning: 'árvore' }],
          mnemonic: 'Três árvores juntas.',
        }
      }
      if (cmd === 'save_kanji_mnemonic') {
        return null
      }
      return null
    })

    const parentInjector = { get: () => null } as unknown as EnvironmentInjector
    const injector = createEnvironmentInjector([], parentInjector)

    runInInjectionContext(injector, () => {
      component = new KanjiBreakdownComponent()
      // Set input signal value
      Object.defineProperty(component, 'kanji', {
        value: () => '森',
      })
    })
  })

  it('should load kanji components and existing mnemonic on init', async () => {
    await component.loadDetails()

    expect(component.components().length).toBe(1)
    expect(component.components()[0].component).toBe('木')
    expect(component.mnemonicText()).toBe('Três árvores juntas.')
  })

  it('should call save_kanji_mnemonic on blurSave', async () => {
    await component.loadDetails()

    component.mnemonicText.set('Nova história mnemónica de teste')
    await component.onBlurSave()

    expect(tauriCore.invoke).toHaveBeenCalledWith('save_kanji_mnemonic', {
      kanji: '森',
      mnemonic: 'Nova história mnemónica de teste',
    })
    expect(component.isSaved()).toBe(true)
  })
})
