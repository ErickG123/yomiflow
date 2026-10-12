import { inject } from '@angular/core'
import { CanActivateFn, Router, UrlTree } from '@angular/router'
import { CoreInitializationService } from '../services/core-initialization.service'

export const onboardingGuard: CanActivateFn = async (): Promise<boolean | UrlTree> => {
  const router = inject(Router)
  const initialization = inject(CoreInitializationService)
  await initialization.initialize()
  const isCompleted = initialization.onboardingComplete()
  return isCompleted === false ? router.createUrlTree(['/onboarding']) : true
}

export const onboardingEntryGuard: CanActivateFn = async (): Promise<boolean | UrlTree> => {
  const router = inject(Router)
  const initialization = inject(CoreInitializationService)
  await initialization.initialize()
  return initialization.onboardingComplete() === true ? router.createUrlTree(['/dashboard']) : true
}
