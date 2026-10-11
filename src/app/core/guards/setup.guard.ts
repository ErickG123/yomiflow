import { inject } from '@angular/core'
import { CanActivateFn, Router } from '@angular/router'
import { invoke } from '@tauri-apps/api/core'

export const setupGuard: CanActivateFn = async () => {
  const router = inject(Router)

  try {
    const isNeeded = await invoke<boolean>('check_onboarding_needed')
    if (isNeeded) {
      return router.createUrlTree(['/onboarding'])
    }
    return true
  } catch (err) {
    console.warn('Failed to verify onboarding status:', err)
    return true
  }
}
