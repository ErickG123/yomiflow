import { Routes } from '@angular/router'

import { setupGuard } from './core/guards/setup.guard'

export const routes: Routes = [
  {
    path: '',
    redirectTo: 'dashboard',
    pathMatch: 'full',
  },
  {
    path: 'onboarding',
    loadComponent: () =>
      import('./features/onboarding/components/onboarding/onboarding.component').then(
        (m) => m.OnboardingComponent
      ),
  },
  {
    path: 'dashboard',
    canActivate: [setupGuard],
    loadComponent: () =>
      import('./features/dashboard/components/dashboard-main/dashboard-main.component').then(
        (m) => m.DashboardMainComponent
      ),
  },
  {
    path: 'dictionary',
    loadComponent: () =>
      import('./features/dictionary/components/dictionary-search/dictionary-search.component').then(
        (m) => m.DictionarySearchComponent
      ),
  },
  {
    path: 'study',
    loadComponent: () =>
      import('./features/srs/components/study-session/study-session.component').then(
        (m) => m.StudySessionComponent
      ),
  },
  {
    path: 'manga',
    loadComponent: () =>
      import('./features/reader/components/manga-library/manga-library.component').then(
        (m) => m.MangaLibraryComponent
      ),
  },
  {
    path: 'reader',
    loadComponent: () =>
      import('./features/reader/components/manga-reader/manga-reader.component').then(
        (m) => m.MangaReaderComponent
      ),
  },
  {
    path: 'video',
    loadComponent: () =>
      import('./features/video/components/video-player/video-player.component').then(
        (m) => m.VideoPlayerComponent
      ),
  },
  {
    path: 'notebook',
    loadComponent: () =>
      import('./features/notebook/components/notebook-editor/notebook-editor.component').then(
        (m) => m.NotebookEditorComponent
      ),
  },
  {
    path: '**',
    redirectTo: 'dashboard',
  },
]
