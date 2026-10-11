import { DatePipe } from '@angular/common'
import { ChangeDetectionStrategy, Component, OnInit, inject, signal } from '@angular/core'
import { TranslocoDirective, TranslocoService } from '@ngneat/transloco'
import { invoke } from '@tauri-apps/api/core'

@Component({
  selector: 'app-cloud-sync',
  standalone: true,
  imports: [DatePipe, TranslocoDirective],
  templateUrl: './cloud-sync.component.html',
  styleUrl: './cloud-sync.component.scss',
  changeDetection: ChangeDetectionStrategy.OnPush,
})
export class CloudSyncComponent implements OnInit {
  private readonly translocoService = inject(TranslocoService)

  // Pre-filled with user's client ID for seamless UX while allowing customization
  readonly clientId = signal<string>(
    '160252445437-pp8aj1i6e0gabnrjan50pum50vddpgti.apps.googleusercontent.com'
  )
  readonly clientSecret = signal<string>('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')

  readonly isAuthenticated = signal<boolean>(false)
  readonly isAuthenticating = signal<boolean>(false)
  readonly isBackingUp = signal<boolean>(false)
  readonly lastBackupTime = signal<Date | null>(null)
  readonly backupSuccessToast = signal<string | null>(null)
  readonly errorMessage = signal<string | null>(null)

  async ngOnInit(): Promise<void> {
    await this.checkAuthStatus()
  }

  async checkAuthStatus(): Promise<void> {
    try {
      const token = await invoke<string | null>('get_setting', {
        key: 'gdrive_access_token',
      })
      this.isAuthenticated.set(!!token)

      const savedClientId = await invoke<string | null>('get_setting', {
        key: 'gdrive_client_id',
      })
      if (savedClientId) {
        this.clientId.set(savedClientId)
      }

      const savedLastBackup = await invoke<string | null>('get_setting', {
        key: 'gdrive_last_backup_time',
      })
      if (savedLastBackup) {
        const secs = parseInt(savedLastBackup, 10)
        if (!isNaN(secs) && secs > 0) {
          this.lastBackupTime.set(new Date(secs * 1000))
        }
      }
    } catch (err) {
      console.warn('Erro ao verificar status de sincronização:', err)
    }
  }

  onClientIdChange(event: Event): void {
    const val = (event.target as HTMLInputElement).value
    this.clientId.set(val)
  }

  onClientSecretChange(event: Event): void {
    const val = (event.target as HTMLInputElement).value
    this.clientSecret.set(val)
  }

  async authenticate(): Promise<void> {
    this.isAuthenticating.set(true)
    this.errorMessage.set(null)

    try {
      await invoke<string>('authenticate_google_drive', {
        clientId: this.clientId().trim(),
        clientSecret: this.clientSecret().trim(),
      })
      this.isAuthenticated.set(true)
      this.backupSuccessToast.set(this.translocoService.translate('cloud.account_connected'))
      setTimeout(() => this.backupSuccessToast.set(null), 4000)
    } catch (err) {
      console.error('Falha na autenticação Google Drive:', err)
      this.errorMessage.set(
        typeof err === 'string' ? err : this.translocoService.translate('cloud.auth_error')
      )
    } finally {
      this.isAuthenticating.set(false)
    }
  }

  async backupNow(): Promise<void> {
    this.isBackingUp.set(true)
    this.errorMessage.set(null)
    this.backupSuccessToast.set(null)

    try {
      const fileId = await invoke<string>('backup_database_to_drive')
      const now = new Date()
      this.lastBackupTime.set(now)
      this.backupSuccessToast.set(
        `${this.translocoService.translate('cloud.backup_success')} (ID: ${fileId})`
      )
      setTimeout(() => this.backupSuccessToast.set(null), 5000)
    } catch (err) {
      console.error('Falha ao enviar backup para o Drive:', err)
      this.errorMessage.set(
        typeof err === 'string' ? err : this.translocoService.translate('cloud.backup_error')
      )
    } finally {
      this.isBackingUp.set(false)
    }
  }

  async disconnect(): Promise<void> {
    try {
      await invoke('set_setting', { key: 'gdrive_access_token', value: '' })
      await invoke('set_setting', { key: 'gdrive_refresh_token', value: '' })
      this.isAuthenticated.set(false)
    } catch (err) {
      console.error('Erro ao desconectar conta:', err)
    }
  }
}
