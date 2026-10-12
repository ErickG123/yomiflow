# YomiFlow

> The Offline-First Japanese Immersion & Second Brain Ecosystem

[![Angular](https://img.shields.io/badge/Angular-22%20Signals-DD0031?style=flat-square&logo=angular&logoColor=white)](https://angular.dev/)
[![Tauri](https://img.shields.io/badge/Tauri-v2.12-24C8D8?style=flat-square&logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![SQLite](https://img.shields.io/badge/SQLite-WAL%20Mode-003B57?style=flat-square&logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![Offline First](https://img.shields.io/badge/Offline--First-100%25-10B981?style=flat-square&logo=speedtest&logoColor=white)](https://github.com/)

---

## Overview

Mastering Japanese through immersion traditionally requires stitching together a fragmented web of separate, cloud-reliant tools: browser pop-up dictionaries, external flashcard programs, cloud sync subscriptions, third-party note editors, and fragile media integrations. When internet access drops or remote APIs shift, the learning workflow breaks.

**YomiFlow** unifies the entire Japanese acquisition lifecycle into a single high-performance, offline-first desktop environment. It seamlessly fuses static and dynamic media consumption (Manga and Video) with knowledge production and retention (Linguistic Notebook and Relational Spaced Repetition). Operating 100% locally with zero cloud dependencies, YomiFlow ensures your private study data, annotations, media corpus, and flashcard history remain strictly yours.

---

## Core Features

### Smart Media Consumption

- **Manga Reader with Zero-Copy Neural OCR:** Open archive packages (`.cbz`, `.zip`, `.cbr`, `.rar`) directly from memory without extracting directories to disk. Box speech bubbles on the page to trigger local vision-to-text inference powered by MangaOCR on ONNX Runtime (`ort`).
- **Video Player with FFmpeg Extraction:** Stream `.mkv` and `.mp4` video locally with sub-millisecond hover dictionary lookups on embedded subtitle streams. Automatically slice native audio clips and extract context screenshots via FFmpeg to generate rich multimedia flashcards with a single click.
- **NHK Web Easy Offline Reader:** Cache articles and native audio clips into local SQLite storage for distraction-free reading with integrated morphological hover breakdown.

### Relational SRS & Gamification

- **Modern Spaced Repetition (FSRS & SM-2):** Advanced memory scheduler built on pure mathematical domain models, tracking card stability and item difficulty across multiple user-defined Decks.
- **Relational Integrity:** Foreign key enforcement (`PRAGMA foreign_keys = ON;`) with cascading deletions between dictionary entries, cards, custom decks, tags, and review logs.
- **Temporal Analytics & Heatmaps:** GitHub-style 16-week consistency heatmaps, review forecast projections, and JLPT coverage metrics (N5 through N1) mapped against dictionary and card corpora.
- **Achievement & Badge Engine:** Persistent milestone awards (e.g., streak milestones, kanji master badges) evaluated locally against review records.

### Global FTS5 OmniSearch

- **One Search Across YomiFlow:** Search SRS vocabulary cards, TipTap notebook notes, and cached NHK articles from the global Ctrl+K palette, with results grouped by source and matching snippets highlighted.
- **SQLite-Powered Index:** A versioned FTS5 index stays synchronized through insert, update, and delete triggers, so search remains available offline without duplicate application-side indexing logic.

### Global Pomodoro & Immersion Tracker

- **Focused Exposure Tracking:** Run global focus and break cycles from the sidebar, categorize time as active reading, passive listening, or SRS review, and receive native desktop notifications when each phase ends.
- **Persistent Immersion Metrics:** Successfully completed focus blocks are stored in local SQLite `study_sessions`, providing a durable measure of study exposure hours by category and date.

### Visual SRS (Image Occlusion)

- **SVG Mask Editor:** Upload or paste manga panels and draw multiple rectangular masks over speech bubbles, kanji, or other visual details.
- **Native Visual Recall:** Review image cards with opaque regions and reveal each hidden area on click, then schedule recall with the existing FSRS ratings while keeping images and mask coordinates local.

### Neuroscience-Based Interleaved SRS

- **Dynamic Component Rendering:** One review queue interleaves vocabulary, image occlusion, grammar notes, and Kana Dojo prompts across decks; Angular switches the card UI by its explicit type and resets response state between cards.
- **Interleaved Practice:** The Rust backend gathers pending reviews and shuffles the combined queue to encourage active recall across different study domains instead of repetitive single-deck sessions.

### Hybrid Linguistic Notebook

- **Notion-Style TipTap Rich-Text:** Clean block and markdown editor with slash command capabilities, split-screen viewing, and autosaving.
- **Real-Time Morphological NLP (Lindera):** Integrated Lindera tokenizer with IPADIC dictionary running on Rust. Parses Japanese text in real time to categorize and syntax-highlight parts of speech (verbs, particles, nouns, adjectives, and auxiliaries) with debounced IPC updates.
- **Offline Grammar Linter:** A debounced Lindera check spots common particle mistakes such as `日本語を分かります`, marks the affected text in TipTap, and offers a one-click `が` correction with an explanation.
- **Morphological Readability Analyzer (Offline MorphMan):** Analyze pasted Japanese text locally with Lindera lemmas crossed against vocabulary cards and FSRS retention in SQLite. A comprehension percentage, average retention metric, and inline unknown-word highlights help estimate how much of a passage is familiar before reading.
- **Strict Content Sanitization:** Comprehensive XSS mitigation utilizing Angular `DomSanitizer` in conjunction with HTML security contexts.

### Kana Dojo

- **Speed Flashcard Drill Engine:** Practice Hiragana and Katakana across Gojūon (base), Dakuon/Handakuon (ten-ten / maru), and Yōon (combination) character sets.
- **Granular Mastery Tracking:** Database persistence (`kana_mastery`) recording character-level accuracy and velocity to power global proficiency metrics on the dashboard.

### Offline-First & BYOS (Bring Your Own Storage)

- **Zero Third-Party Cloud Lock-In:** No remote proprietary telemetry or forced cloud backends.
- **Native Google Drive Backup:** Direct synchronization of the local SQLite database to the user's personal Google Drive using native OAuth2 (`oauth2`, `tiny_http`, and `reqwest`). Users retain absolute ownership and privacy of their data.
- **Discord Rich Presence (Zen Mode):** Optional desktop status broadcast via `discord-rich-presence` displaying real-time immersion sessions and study duration.

---

## Architecture & Technology Stack

### SQLite Schema Migrations

The Rust backend manages SQLite schema evolution through versioned migrations in `src-tauri/src/database/migrations.rs`. At startup, it creates `schema_migrations`, applies each unapplied migration in its own transaction, and records the version only after its SQL succeeds. A failed migration rolls back and stops database initialization so the app cannot continue with a partially updated schema. The checked-in `schema.sql` defines the current base schema; later migrations handle compatibility changes for existing databases.

```
+------------------------------------------------------------------------+
|                          Angular 22 Frontend                           |
|       Signals State  *  Tailwind CSS (Zinc)  *  Transloco i18n         |
|         TipTap Markdown Editor  *  Feature-Based Modular Design        |
+------------------------------------------------------------------------+
                                    |
                          Tauri v2 IPC Layer
                     (Strictly Typed Rust Commands)
                                    |
+------------------------------------------------------------------------+
|                         Rust Core Application                          |
|  * Domain-Driven Design (Pure Domain Logic vs Application Adapters)    |
|  * Asynchronous Runtime (Tokio)                                        |
|  * Heavy I/O & Compute in `tokio::task::spawn_blocking`                |
|    - Lindera NLP Tokenization                                          |
|    - ONNX Runtime MangaOCR Inference                                   |
|    - Archive & FFmpeg Media Processing                                 |
+------------------------------------------------------------------------+
                                    |
+------------------------------------------------------------------------+
|                         SQLite Storage Layer                           |
|  * `PRAGMA journal_mode = WAL` (Concurrent Read/Write Performance)     |
|  * `PRAGMA foreign_keys = ON` (Cascading Referential Integrity)        |
|  * Prepared Statements Exclusively (Parameterized SQLi Immunity)       |
+------------------------------------------------------------------------+
```

### IPC Bridge Pattern

The frontend interacts with the native runtime exclusively through typed IPC invocations using `@tauri-apps/api/core`. Computationally intensive tasks—such as morphological tokenization, OCR tensor evaluation, archive decompression, and database batch imports—are offloaded from the UI and event loop to worker threads via `tokio::task::spawn_blocking`.

---

## Project Structure

The project follows a clean separation between feature-driven UI modules, reusable components, and native Rust domain services:

```
yomiflow/
|-- .agents/                          # Architectural guidelines, context, and agent rules
|   |-- rules.md                      # Engineering standards, testing, and formatting rules
|   |-- context_gamification.md       # Temporal aggregation, streaks, and heatmap specs
|   |-- context_nlp_notebook.md       # Lindera tokenization and TipTap notebook specs
|   |-- context_srs_relational.md     # Relational SQLite SRS and cascade delete specs
|-- src/                              # Angular 22 Frontend Application
|   |-- app/
|   |   |-- core/                     # Application singletons, guards, and root services
|   |   |-- features/                 # Feature-based business modules
|   |   |   |-- calendar/             # Temporal review forecast and heatmap calendar
|   |   |   |-- dashboard/            # Proficiency analytics and quick-action hub
|   |   |   |-- decks/                # Deck management, tag assignments, and card stats
|   |   |   |-- dictionary/           # Yomichan entry search and radical breakdown
|   |   |   |-- kana/                 # Kana Dojo speed training and mastery tracking
|   |   |   |-- notebook/             # TipTap rich-text editor with Lindera NLP highlighting
|   |   |   |-- reader/               # Manga reader (.cbz), web news, and EPUB viewer
|   |   |   |-- settings/             # System preferences and Google Drive BYOS sync
|   |   |   |-- shadowing/            # Audio transcription and shadowing drill room
|   |   |   |-- srs/                  # FSRS / SM-2 flashcard study session
|   |   |   |-- stats/                # JLPT analytics and progress charts
|   |   |   |-- video/                # Video player with subtitle hover dictionary
|   |   |-- shared/                   # Generic UI components, pipes, and directives
|-- src-tauri/                        # Tauri v2 Backend & Rust Core
|   |-- src/
|   |   |-- database/                 # Versioned SQLite migrations and base schema
|   |   |-- commands/                 # Tauri IPC command entry points
|   |   |   |-- ai_ocr.rs             # ONNX Runtime MangaOCR pipeline
|   |   |   |-- dictionary.rs         # Yomichan SQLite indexed lookups
|   |   |   |-- drive_sync.rs         # OAuth2 Google Drive synchronization
|   |   |   |-- kana.rs               # Kana Dojo mastery score persistence
|   |   |   |-- news.rs               # NHK News Web Easy offline scraper and cache
|   |   |   |-- nlp.rs                # Lindera morphological analysis adapter
|   |   |   |-- reader.rs             # In-memory archive extraction and EPUB handling
|   |   |   |-- srs.rs                # FSRS calculation, deck routing, and reviews
|   |   |   |-- stats.rs              # Heatmap aggregations and JLPT coverage
|   |   |   |-- video.rs              # FFmpeg subtitle and audio extraction
|   |   |-- db/                       # Schema migrations and seed initializers
|   |   |-- domain/                   # Pure business logic (SRS math, NLP tagging, OCR)
|   |   |-- models/                   # Rust struct definitions and data transfer objects
|   |-- Cargo.toml                    # Rust dependencies and compiler configuration
|-- package.json                      # Frontend dependencies and npm build scripts
```

---

## Getting Started

### Prerequisites

Ensure the following runtimes and tools are installed on your workstation:

1. **Node.js**: v20+ with `npm`
2. **Rust & Cargo**: v1.90+ (via [rustup.rs](https://rustup.rs/))
3. **Tauri System Dependencies**:
   - **Linux**: `libwebkit2gtk-4.1-dev`, `build-essential`, `curl`, `wget`, `file`, `libssl-dev`, `libayatana-appindicator3-dev`, `librsvg2-dev`
   - **macOS**: Xcode Command Line Tools
   - **Windows**: Microsoft C++ Build Tools and WebView2 (preinstalled on Windows 10/11)
4. **FFmpeg** _(Optional, recommended)_: Required for subtitle extraction and audio clipping from `.mkv`/`.mp4` media files.

### Installation & Development

1. **Clone the Repository:**

   ```bash
   git clone https://github.com/ErickG123/yomiflow.git
   cd yomiflow
   ```

2. **Install Frontend Dependencies:**

   ```bash
   npm install
   ```

3. **Launch in Development Mode:**
   ```bash
   npm run tauri dev
   ```

## Contributing & Development Workflow

Use [Conventional Commits](https://www.conventionalcommits.org/) for every commit. Commit messages must use a recognized type and description, such as `feat: add study shortcut`, `fix: correct sidebar overflow`, or `chore: update dependencies`. Commitlint validates each message.

Husky runs local checks before commits: lint-staged formats staged Angular TypeScript, HTML, and SCSS with Prettier and applies ESLint fixes to TypeScript. The pre-commit hook also checks Rust formatting with `cargo fmt` and runs Clippy with warnings treated as errors.

---

## Quality Assurance & Verification

YomiFlow enforces strict quality gates across both frontend and backend codebases:

```bash
# Frontend: Run ESLint inspection
npm run lint

# Frontend: Run Unit & Component tests (Vitest)
npm run test:ci

# Frontend: Compile Angular production bundle
npm run build

# Backend: Verify Rust formatting
cargo fmt -- --check --manifest-path src-tauri/Cargo.toml

# Backend: Run Clippy static analysis with zero warnings allowed
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Backend: Execute Rust test suite (SQLite in-memory test coverage)
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## License

Distributed under the MIT License. See `LICENSE` for more information.
