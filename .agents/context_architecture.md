# Angular Architecture and Loading Boundaries

## Application hierarchy

`App` (`src/app/app.ts`) is the standalone bootstrap component. Its template contains only `<router-outlet />`; application chrome and route-specific presentation live in layouts.

- `OnboardingLayoutComponent` owns the focused, sidebar-free outlet for `/onboarding`.
- `MainLayoutComponent` owns the application shell: navigation sidebar, header and settings, Pomodoro/Zen controls, processing overlay, and the outlet for the main ecosystem.
- Feature components are rendered beneath the main layout and do not need to recreate global navigation.

The onboarding route is matched before the empty main-layout route. The shared onboarding state is loaded by `CoreInitializationService`; `onboardingEntryGuard` keeps completed users out of onboarding, and `onboardingGuard` redirects incomplete users from ecosystem routes. Both await the same idempotent initialization promise, so they do not race the router's initial navigation.

## Bootstrap responsibilities

`app.config.ts` groups global providers: router with component input binding, HttpClient, Transloco, global error listeners, and `APP_INITIALIZER`. The initializer invokes `CoreInitializationService`, which checks the onboarding state and registers the backend progress listener. Notification permission and the optional startup notification run asynchronously so the permission prompt does not hold the Angular bootstrap screen open. Feature services remain `providedIn: 'root'` on their own declarations and are not listed in the application provider array.

## Lazy loading pattern

The root `app.routes.ts` references feature code only through dynamic `import()` callbacks. Each feature screen uses `loadComponent`; features with multiple paths export a `Routes` array from their own route file and are attached through `loadChildren`. Reader paths live in `features/reader/reader.routes.ts`; deck paths live in `features/decks/decks.routes.ts`. Expensive Notebook, Manga/EPUB Reader, SRS, and Dictionary components are therefore absent from the initial route component graph and load when their route is activated.

When adding another sub-route to a feature, keep that route definition in the feature's `*.routes.ts` file and connect the file using `loadChildren`. Avoid importing page components into `app.routes.ts` or the root bootstrap component.

## Global audio player

The global player is mounted by `MainLayoutComponent` beside the route outlet, so navigating between feature routes does not destroy its UI. `GlobalAudioPlayerService` is provided in root and owns playback Signals, the current track, volume, queue, and listening history. Commands go through Tauri IPC to a process-wide Rust audio runtime; that runtime owns Rodio's output stream and sink independently of Angular component lifetimes. The Rust tracker increments `passive_listening_log.listened_ms` while the sink is actively playing. Migration 4 also creates `audio_library`, where imported file paths, ID3 metadata, cover art and duration persist. `/audio-vault` is lazy-loaded and reads the same root service state to manage the queue, library, and SQLite history.
