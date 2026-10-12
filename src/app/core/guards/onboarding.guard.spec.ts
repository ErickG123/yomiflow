import {
  createEnvironmentInjector,
  EnvironmentInjector,
  runInInjectionContext,
  signal,
} from '@angular/core'
import { Router } from '@angular/router'
import { onboardingEntryGuard, onboardingGuard } from './onboarding.guard'
import { CoreInitializationService } from '../services/core-initialization.service'

describe('onboarding guards', () => {
  const onboardingComplete = signal<boolean | null>(null)
  const initialize = vi.fn(async () => undefined)
  const redirect = { redirect: true }
  const router = { createUrlTree: vi.fn(() => redirect) }
  const parent = { get: () => null } as unknown as EnvironmentInjector

  function runGuard(
    guard: typeof onboardingGuard | typeof onboardingEntryGuard,
  ): Promise<boolean | object> {
    initialize.mockClear()
    router.createUrlTree.mockClear()
    const injector = createEnvironmentInjector(
      [
        { provide: Router, useValue: router },
        {
          provide: CoreInitializationService,
          useValue: { initialize, onboardingComplete },
        },
      ],
      parent,
    )
    return runInInjectionContext(injector, () => guard()) as Promise<boolean | object>
  }

  it('redirects protected routes to onboarding when setup is incomplete', async () => {
    onboardingComplete.set(false)
    const result = await runGuard(onboardingGuard)

    expect(initialize).toHaveBeenCalledOnce()
    expect(router.createUrlTree).toHaveBeenCalledWith(['/onboarding'])
    expect(result).toBe(redirect)
  })

  it('allows protected routes when setup is complete', async () => {
    onboardingComplete.set(true)
    const result = await runGuard(onboardingGuard)

    expect(result).toBe(true)
    expect(router.createUrlTree).not.toHaveBeenCalled()
  })

  it('redirects completed users away from onboarding', async () => {
    onboardingComplete.set(true)
    const result = await runGuard(onboardingEntryGuard)

    expect(router.createUrlTree).toHaveBeenCalledWith(['/dashboard'])
    expect(result).toBe(redirect)
  })
})
