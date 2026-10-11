import { describe, it, expect, vi, beforeEach } from 'vitest'
import { AiModelService } from './ai-model.service'
import * as tauriCore from '@tauri-apps/api/core'
import * as tauriEvent from '@tauri-apps/api/event'

vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(),
}))

describe('AiModelService', () => {
  let service: AiModelService

  beforeEach(() => {
    vi.clearAllMocks()
    service = new AiModelService()
  })

  it('should initialize with default idle signals', () => {
    expect(service.isDownloading()).toBe(false)
    expect(service.downloadProgress()).toBe(0)
    expect(service.isModelReady()).toBe(false)
  })

  it('should return true and set isModelReady if model already exists on disk', async () => {
    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'check_model_exists') return true
      return false
    })

    const ready = await service.ensureModelExists()

    expect(ready).toBe(true)
    expect(service.isModelReady()).toBe(true)
    expect(service.isDownloading()).toBe(false)
  })

  it('should track download progress and complete via event listeners when model is missing', async () => {
    let progressCallback: ((event: { payload: number }) => void) | null = null
    let completeCallback: (() => void) | null = null

    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'check_model_exists') return false
      if (cmd === 'download_ai_model') {
        // Simulate progress events fired from Tauri backend
        if (progressCallback) progressCallback({ payload: 45 })
        if (progressCallback) progressCallback({ payload: 90 })
        if (completeCallback) completeCallback()
        return undefined
      }
      return undefined
    })

    vi.mocked(tauriEvent.listen).mockImplementation(async (eventName: string, handler: unknown) => {
      if (eventName === 'model-download-progress') {
        progressCallback = handler as (event: { payload: number }) => void
      }
      if (eventName === 'model-download-complete') {
        completeCallback = handler as () => void
      }
      return () => {}
    })

    const ensurePromise = service.ensureModelExists()
    const success = await ensurePromise

    expect(success).toBe(true)
    expect(service.downloadProgress()).toBe(100)
    expect(service.isDownloading()).toBe(false)
    expect(service.isModelReady()).toBe(true)
  })

  it('should reset download state if download fails', async () => {
    vi.mocked(tauriCore.invoke).mockImplementation(async (cmd: string) => {
      if (cmd === 'check_model_exists') return false
      if (cmd === 'download_ai_model') throw new Error('Download network failure')
      return undefined
    })
    vi.mocked(tauriEvent.listen).mockResolvedValue(() => {})

    const success = await service.ensureModelExists()

    expect(success).toBe(false)
    expect(service.isDownloading()).toBe(false)
    expect(service.isModelReady()).toBe(false)
  })
})
