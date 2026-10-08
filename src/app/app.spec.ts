import { describe, it, expect } from 'vitest'
import { runInInjectionContext, EnvironmentInjector, createEnvironmentInjector } from '@angular/core'
import { App } from './app'

describe('App', () => {
  it('should initialize successfully', () => {
    const parentInjector = { get: () => null } as unknown as EnvironmentInjector
    const injector = createEnvironmentInjector([], parentInjector)

    runInInjectionContext(injector, () => {
      const component = new App()
      expect(component).toBeTruthy()
      expect(component.availableLangs.length).toBe(3)
      expect(component.activeLang()).toBe('en')
    })
  })
})
