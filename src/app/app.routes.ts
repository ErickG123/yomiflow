import { Routes } from '@angular/router'

import { onboardingGuard } from './core/guards/onboarding.guard'

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
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/dashboard/components/dashboard-main/dashboard-main.component').then(
        (m) => m.DashboardMainComponent
      ),
  },
  {
    path: 'dictionary',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/dictionary/components/dictionary-search/dictionary-search.component').then(
        (m) => m.DictionarySearchComponent
      ),
  },
  {
    path: 'study',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/srs/components/study-session/study-session.component').then(
        (m) => m.StudySessionComponent
      ),
  },
  {
    path: 'image-occlusion',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/image-occlusion/components/image-occlusion/image-occlusion.component').then(
        (m) => m.ImageOcclusionComponent
      ),
  },
  {
    path: 'kana',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/kana/components/kana-trainer/kana-trainer.component').then(
        (m) => m.KanaTrainerComponent
      ),
  },
  {
    path: 'manga',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/reader/components/manga-library/manga-library.component').then(
        (m) => m.MangaLibraryComponent
      ),
  },
  {
    path: 'reader',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/reader/components/manga-reader/manga-reader.component').then(
        (m) => m.MangaReaderComponent
      ),
  },
  {
    path: 'web',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/reader/components/web-reader/web-reader.component').then(
        (m) => m.WebReaderComponent
      ),
  },
  {
    path: 'epub',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/reader/components/epub-reader/epub-reader.component').then(
        (m) => m.EpubReaderComponent
      ),
  },
  {
    path: 'video',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/video/components/video-player/video-player.component').then(
        (m) => m.VideoPlayerComponent
      ),
  },
  {
    path: 'shadowing',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/shadowing/components/shadowing-trainer/shadowing-trainer.component').then(
        (m) => m.ShadowingTrainerComponent
      ),
  },
  {
    path: 'notebook',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/notebook/components/notebook-editor/notebook-editor.component').then(
        (m) => m.NotebookEditorComponent
      ),
  },
  {
    path: 'decks',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/decks/components/deck-list/deck-list.component').then(
        (m) => m.DeckListComponent
      ),
  },
  {
    path: 'decks/:id/settings',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/decks/components/deck-settings/deck-settings.component').then(
        (m) => m.DeckSettingsComponent
      ),
  },
  {
    path: 'calendar',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/calendar/components/calendar-view/calendar-view.component').then(
        (m) => m.CalendarViewComponent
      ),
  },
  {
    path: 'analytics',
    canActivate: [onboardingGuard],
    loadComponent: () =>
      import('./features/stats/components/jlpt-analytics/jlpt-analytics.component').then(
        (m) => m.JlptAnalyticsComponent
      ),
  },
  {
    path: '**',
    redirectTo: 'dashboard',
  },
]
