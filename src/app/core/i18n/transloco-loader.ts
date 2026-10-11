import { inject, Injectable } from '@angular/core'
import { Translation, TranslocoLoader } from '@ngneat/transloco'
import { HttpClient } from '@angular/common/http'
import { catchError, of } from 'rxjs'

@Injectable({ providedIn: 'root' })
export class TranslocoHttpLoader implements TranslocoLoader {
  private readonly http = inject(HttpClient)

  getTranslation(lang: string) {
    // Relative path ./assets/i18n/${lang}.json for resilient Tauri webview asset resolution
    return this.http.get<Translation>(`./assets/i18n/${lang}.json`).pipe(
      catchError((err) => {
        console.warn(`[TranslocoLoader] Falha ao carregar ficheiro ./assets/i18n/${lang}.json:`, err)
        // Fallback gracioso: tenta inglês se a língua não for inglês, ou objeto vazio
        if (lang !== 'en') {
          return this.http.get<Translation>(`./assets/i18n/en.json`).pipe(
            catchError((fallbackErr) => {
              console.warn('[TranslocoLoader] Fallback para en.json também falhou:', fallbackErr)
              return of({} as Translation)
            })
          )
        }
        return of({} as Translation)
      })
    )
  }
}
