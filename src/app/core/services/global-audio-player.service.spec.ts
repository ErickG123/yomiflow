import { describe, expect, it, vi } from 'vitest'
import { GlobalAudioPlayerService } from './global-audio-player.service'
import * as tauriCore from '@tauri-apps/api/core'

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }))
vi.mock('@tauri-apps/plugin-dialog', () => ({ open: vi.fn() }))

describe('GlobalAudioPlayerService', () => {
  it('formats playback times as minutes and seconds', () => {
    const service = new GlobalAudioPlayerService()
    expect(service.formatTime(0)).toBe('0:00')
    expect(service.formatTime(125_000)).toBe('2:05')
  })

  it('clamps volume and seek requests before invoking Tauri', async () => {
    const service = new GlobalAudioPlayerService()
    service.track.set({
      path: 'local.mp3',
      title: 'Track',
      artist: 'Artist',
      album: 'Album',
      cover_data_url: null,
      duration_ms: 60_000,
    })

    await service.setVolume(1.5)
    await service.seek(-500)

    expect(service.volume()).toBe(1)
    expect(service.positionMs()).toBe(0)
    expect(tauriCore.invoke).toHaveBeenNthCalledWith(1, 'set_volume', { volume: 1 })
    expect(tauriCore.invoke).toHaveBeenNthCalledWith(2, 'seek_to', { positionMs: 0 })
  })
})
