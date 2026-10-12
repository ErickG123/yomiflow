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

## Supreme code audit — 2026-10-12

- [x] **DDD, KISS, DRY and YAGNI:** The FTS5 query adapter now lives in `db/search.rs`, while its Tauri command only obtains state and delegates. The interleaving algorithm and its test moved from the command to pure `domain/interleaving.rs`. Removed `calculate_srs_step` from the SRS command because it had no production callers; production review scheduling already uses `domain/fsrs.rs`. Remaining work: some command modules still combine SQL persistence with use-case orchestration, notably interleaved queue loading and SRS review transactions. This audit did not claim those files are adapter-only.
- [x] **Angular boundaries:** New Audio Vault UI strings use Transloco and the route is exposed from the sidebar. The root audio player continues to use a root-provided signal service. Existing components remain oversized: Notebook Editor (671 lines), Manga Reader (385), EPUB Reader (279), Calendar (278), Kana Trainer (225), and Image Occlusion (216), among others. Extracting all of those in this audit would require focused behavior-preserving refactors; they are recorded as follow-up work.
- [x] **Strict models and file separation:** A repository scan found no interface/type/enum declarations in `.component.ts`, `.service.ts`, or `.guard.ts`, and no inline `template`/`styles` properties in Angular components. Models remain in dedicated model files.
- [x] **XSS and SQLite:** `SafeHtmlPipe` and EPUB rendering sanitize with Angular `SecurityContext.HTML`. Furigana output is now sanitized in the pipe before reaching `[innerHTML]`; the Rust generator also escapes surface and reading text. SQLite statements bind query values, and the audit removed formatted SQL from migration execution and column inspection. Schema changes remain versioned transactional migrations.
- [x] **QA:** FSRS and Lindera have isolated unit tests; integration tests use `Connection::open_in_memory()` and cover migrations, cascade deletes, and FTS triggers. Angular specs use Vitest with mocked Tauri APIs where tested; this repository does not use Karma. Broader coverage remains incomplete and is listed as follow-up work. The current machine ran out of C: drive space during this audit, so a fresh full Rust test run could not be completed.
- [x] **Responsive UI and i18n:** The three locale JSON files were compared recursively and have identical keys. New audio controls and page strings use Transloco. A scan still finds hardcoded labels in older screens (for example dictionary loading/empty states and the command palette empty state); those are UI localization follow-ups, not missing locale keys. Existing layouts generally use bounded flex/grid containers; individual legacy pages need visual checks at narrow widths.
- [x] **Hooks:** `.husky/pre-commit` runs `lint-staged`, `cargo fmt --all -- --check`, and `cargo clippy -- -D warnings`; `.husky/commit-msg` runs Conventional Commit validation. The hook commands and `lint-staged` configuration are present.

Validation evidence for the audit: `npm run lint`, `npm run test:ci` (25 files, 82 tests), and `npm run build` passed after the Angular changes. Recursive key comparison confirmed `en.json`, `pt.json`, and `es.json` have the same structure; `cargo fmt --all -- --check` passed. `cargo clippy -- -D warnings` had passed earlier in the audit before the final domain extraction; the later Clippy and `cargo check --tests` attempts were blocked because C: had no free space and the linker could not create generated DLLs. Re-run Rust Clippy and tests after freeing disk space.
