export interface CloudSyncState {
  isAuthenticated: boolean
  clientId: string
  clientSecret: string
  isAuthenticating: boolean
  isBackingUp: boolean
  lastBackupTime: string | null
  backupSuccessMessage: string | null
  errorMessage: string | null
}
