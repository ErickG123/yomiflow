import { Routes } from '@angular/router'
import { onboardingEntryGuard, onboardingGuard } from './core/guards/onboarding.guard'

export const routes: Routes = [
  {
    path: 'onboarding',
    canActivate: [onboardingEntryGuard],
    loadComponent: () =>
      import('./layouts/onboarding-layout/onboarding-layout.component').then(
        (module) => module.OnboardingLayoutComponent,
      ),
    children: [
      {
        path: '',
        loadComponent: () =>
          import('./features/onboarding/components/onboarding/onboarding.component').then(
            (module) => module.OnboardingComponent,
          ),
      },
    ],
  },
  {
    path: '',
    loadComponent: () =>
      import('./layouts/main-layout/main-layout.component').then(
        (module) => module.MainLayoutComponent,
      ),
    canActivateChild: [onboardingGuard],
    children: [
      { path: '', redirectTo: 'dashboard', pathMatch: 'full' },
      {
        path: 'dashboard',
        loadComponent: () =>
          import('./features/dashboard/components/dashboard-main/dashboard-main.component').then(
            (module) => module.DashboardMainComponent,
          ),
      },
      {
        path: 'dictionary',
        loadComponent: () =>
          import('./features/dictionary/components/dictionary-search/dictionary-search.component').then(
            (module) => module.DictionarySearchComponent,
          ),
      },
      {
        path: 'readability',
        loadComponent: () =>
          import('./features/readability/components/readability-analyzer/readability-analyzer.component').then(
            (module) => module.ReadabilityAnalyzerComponent,
          ),
      },
      {
        path: 'study',
        loadComponent: () =>
          import('./features/srs/components/study-session/study-session.component').then(
            (module) => module.StudySessionComponent,
          ),
      },
      {
        path: 'image-occlusion',
        loadComponent: () =>
          import('./features/image-occlusion/components/image-occlusion/image-occlusion.component').then(
            (module) => module.ImageOcclusionComponent,
          ),
      },
      {
        path: 'kana',
        loadComponent: () =>
          import('./features/kana/components/kana-trainer/kana-trainer.component').then(
            (module) => module.KanaTrainerComponent,
          ),
      },
      {
        path: '',
        loadChildren: () =>
          import('./features/reader/reader.routes').then((module) => module.readerRoutes),
      },
      {
        path: 'decks',
        loadChildren: () =>
          import('./features/decks/decks.routes').then((module) => module.deckRoutes),
      },
      {
        path: 'video',
        loadComponent: () =>
          import('./features/video/components/video-player/video-player.component').then(
            (module) => module.VideoPlayerComponent,
          ),
      },
      {
        path: 'shadowing',
        loadComponent: () =>
          import('./features/shadowing/components/shadowing-trainer/shadowing-trainer.component').then(
            (module) => module.ShadowingTrainerComponent,
          ),
      },
      {
        path: 'audio-vault',
        loadComponent: () =>
          import('./features/audio-vault/audio-vault.component').then(
            (module) => module.AudioVaultComponent,
          ),
      },
      {
        path: 'notebook',
        loadComponent: () =>
          import('./features/notebook/components/notebook-editor/notebook-editor.component').then(
            (module) => module.NotebookEditorComponent,
          ),
      },
      {
        path: 'calendar',
        loadComponent: () =>
          import('./features/calendar/components/calendar-view/calendar-view.component').then(
            (module) => module.CalendarViewComponent,
          ),
      },
      {
        path: 'analytics',
        loadComponent: () =>
          import('./features/stats/components/jlpt-analytics/jlpt-analytics.component').then(
            (module) => module.JlptAnalyticsComponent,
          ),
      },
      { path: '**', redirectTo: 'dashboard' },
    ],
  },
  { path: '**', redirectTo: 'dashboard' },
]
