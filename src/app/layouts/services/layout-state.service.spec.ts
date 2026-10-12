import { LayoutStateService } from './layout-state.service'

describe('LayoutStateService', () => {
  it('toggles settings state without changing sidebar state', () => {
    const service = new LayoutStateService()

    service.toggleSettings()
    expect(service.isSettingsOpen()).toBe(true)
    expect(service.isSidebarOpen()).toBe(false)

    service.toggleSettings()
    expect(service.isSettingsOpen()).toBe(false)
  })
})
