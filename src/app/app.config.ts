import {
  APP_INITIALIZER,
  ApplicationConfig,
  isDevMode,
  provideBrowserGlobalErrorListeners,
} from '@angular/core'
import { provideRouter, withComponentInputBinding } from '@angular/router'
import { provideHttpClient } from '@angular/common/http'
import { provideTransloco } from '@ngneat/transloco'

import { routes } from './app.routes'
import { TranslocoHttpLoader } from './core/i18n/transloco-loader'
import { CoreInitializationService } from './core/services/core-initialization.service'

function initializeCore(coreInitialization: CoreInitializationService): () => Promise<void> {
  return () => coreInitialization.initialize()
}

export const appConfig: ApplicationConfig = {
  providers: [
    provideBrowserGlobalErrorListeners(),
    provideRouter(routes, withComponentInputBinding()),
    provideHttpClient(),
    provideTransloco({
      config: {
        availableLangs: ['en', 'pt', 'es', 'ja'],
        defaultLang: 'en',
        fallbackLang: 'en',
        reRenderOnLangChange: true,
        prodMode: !isDevMode(),
        missingHandler: { logMissingKey: isDevMode() },
      },
      loader: TranslocoHttpLoader,
    }),
    {
      provide: APP_INITIALIZER,
      useFactory: initializeCore,
      deps: [CoreInitializationService],
      multi: true,
    },
  ],
}
