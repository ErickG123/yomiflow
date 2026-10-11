import { describe, it, expect, vi, beforeEach } from 'vitest'
import { CloudSyncComponent } from './cloud-sync.component'
import * as tauriCore from '@tauri-apps/api/core'
import { EnvironmentInjector, createEnvironmentInjector, runInInjectionContext } from '@angular/core'
import { TranslocoService } from '@ngneat/transloco'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

describe('CloudSyncComponent', () => {
  let component: CloudSyncComponent
  let translocoMock: Partial<TranslocoService>

  beforeEach(() => {
    vi.restoreAllMocks()

    translocoMock = {
      translate: vi.fn((key: string) => {
        if (key === 'cloud.account_connected') return 'Conta Google conectada com sucesso!'
        if (key === 'cloud.backup_success') return 'Backup concluído com sucesso no Google Drive!'
        return key
      }),
    }

    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'get_setting') {
        return null
      }
      return null
    })

    const providers = [{ provide: TranslocoService, useValue: translocoMock }]
    const parentInjector = {
      get: (token: unknown) => providers.find((p) => p.provide === token)?.useValue ?? null,
    } as unknown as EnvironmentInjector
    const injector = createEnvironmentInjector([], parentInjector)

    runInInjectionContext(injector, () => {
      component = new CloudSyncComponent()
    })
  })

  it('should initialize with disconnected state by default', () => {
    expect(component.isAuthenticated()).toBe(false)
  })

  it('should become authenticated when authenticate succeeds', async () => {
    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'authenticate_google_drive') {
        return 'Sucesso'
      }
      return null
    })

    await component.authenticate()
    expect(component.isAuthenticated()).toBe(true)
    expect(component.backupSuccessToast()).toContain('conectada com sucesso')
  })

  it('should trigger backup_database_to_drive when backupNow is called', async () => {
    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'backup_database_to_drive') {
        return 'file-12345'
      }
      return null
    })

    await component.backupNow()
    expect(component.backupSuccessToast()).toContain('file-12345')
    expect(component.lastBackupTime()).not.toBeNull()
  })
})
