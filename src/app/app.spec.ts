import { describe, it, expect } from 'vitest'
import { App } from './app'

describe('App', () => {
  it('should create the root router shell', () => {
    expect(new App()).toBeTruthy()
  })
})
