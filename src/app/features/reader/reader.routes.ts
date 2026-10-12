import { Routes } from '@angular/router'

export const readerRoutes: Routes = [
  {
    path: 'manga',
    loadComponent: () =>
      import('./components/manga-library/manga-library.component').then(
        (module) => module.MangaLibraryComponent,
      ),
  },
  {
    path: 'reader',
    loadComponent: () =>
      import('./components/manga-reader/manga-reader.component').then(
        (module) => module.MangaReaderComponent,
      ),
  },
  {
    path: 'web',
    loadComponent: () =>
      import('./components/web-reader/web-reader.component').then(
        (module) => module.WebReaderComponent,
      ),
  },
  {
    path: 'epub',
    loadComponent: () =>
      import('./components/epub-reader/epub-reader.component').then(
        (module) => module.EpubReaderComponent,
      ),
  },
]
