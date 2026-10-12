import { describe, it, expect, vi } from 'vitest'
import { ZenModeService } from './zen-mode.service'

const tauriWindow = vi.hoisted(() => ({ setFullscreen: vi.fn().mockResolvedValue(undefined) }))
vi.mock('@tauri-apps/api/window', () => ({
  getCurrentWindow: () => tauriWindow,
}))
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}))

describe('ZenModeService', () => {
  it('should initialize with default states', () => {
    const service = new ZenModeService()
    expect(service.isZenMode()).toBe(false)
    expect(service.currentMode()).toBe('work')
    expect(service.timeLeft()).toBe(25 * 60)
    expect(service.formattedTime()).toBe('25:00')
  })

  it('should toggle zen mode and format timer', async () => {
    const service = new ZenModeService()
    await service.toggleZenMode()
    expect(service.isZenMode()).toBe(true)

    service.switchMode('break')
    expect(service.currentMode()).toBe('break')
    expect(service.formattedTime()).toBe('05:00')

    await service.exitZenMode()
    expect(service.isZenMode()).toBe(false)
  })

  it('should reset timer properly', () => {
    const service = new ZenModeService()
    service.switchMode('work')
    service.resetTimer()
    expect(service.timeLeft()).toBe(25 * 60)
    expect(service.isTimerRunning()).toBe(false)
  })
})
