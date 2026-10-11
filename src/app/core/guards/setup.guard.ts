import { inject } from '@angular/core'
import { CanActivateFn, Router } from '@angular/router'
import { invoke } from '@tauri-apps/api/core'

export const setupGuard: CanActivateFn = async () => {
  const router = inject(Router)

  try {
    const isCompleted = await invoke<boolean>('has_completed_onboarding')
    if (!isCompleted) {
      return router.createUrlTree(['/onboarding'])
    }
    return true
  } catch (err) {
    console.warn('Failed to verify onboarding status:', err)
    return true
  }
}
