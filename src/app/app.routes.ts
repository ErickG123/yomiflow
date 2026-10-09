import { Routes } from '@angular/router'
import { DashboardMainComponent } from './features/dashboard/components/dashboard-main/dashboard-main.component'
import { DictionarySearchComponent } from './features/dictionary/components/dictionary-search/dictionary-search.component'
import { StudySessionComponent } from './features/srs/components/study-session/study-session.component'

export const routes: Routes = [
  {
    path: '',
    redirectTo: 'dashboard',
    pathMatch: 'full',
  },
  {
    path: 'dashboard',
    component: DashboardMainComponent,
  },
  {
    path: 'dictionary',
    component: DictionarySearchComponent,
  },
  {
    path: 'study',
    component: StudySessionComponent,
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
    path: '**',
    redirectTo: 'dashboard',
  },
]
