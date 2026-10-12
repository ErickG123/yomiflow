import { Routes } from '@angular/router'

export const deckRoutes: Routes = [
  {
    path: '',
    loadComponent: () =>
      import('./components/deck-list/deck-list.component').then(
        (module) => module.DeckListComponent,
      ),
  },
  {
    path: ':id/settings',
    loadComponent: () =>
      import('./components/deck-settings/deck-settings.component').then(
        (module) => module.DeckSettingsComponent,
      ),
  },
]
