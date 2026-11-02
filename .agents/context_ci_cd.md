# CI/CD and Automated Release Rules

> **CRITICAL DIRECTIVE:** Keep workflow behavior, commit conventions, and these release rules aligned. Do not create release tags manually; the GitHub Actions release workflow owns version tagging and publication.

## Continuous Integration (CI)

The CI workflow runs for pull requests targeting `main` and pushes to `main`. A failed quality gate blocks the workflow from succeeding. It checks Prettier formatting on changed frontend TypeScript, HTML, and SCSS files, runs ESLint, executes Vitest with the V8 coverage provider, checks Rust formatting and Clippy, runs Rust tests with `cargo-llvm-cov`, and builds the Angular production bundle. Both frontend and Rust coverage commands must complete and generate their reports; no numeric coverage threshold is currently configured.

### Build and coverage platform constraints

- CI and Linux release packaging use `ubuntu-24.04`. Rust coverage links ONNX Runtime C++ objects from `ort-sys`; Ubuntu 22.04 failed to link the instrumented tests due to unresolved C++ standard-library and glibc symbols (including `__isoc23_*`). Keep this runner unless the ONNX Runtime toolchain is changed and verified elsewhere.
- macOS release builds target `aarch64-apple-darwin` only. The configured `ort-sys` version has a prebuilt ONNX Runtime for Apple Silicon but not `x86_64-apple-darwin`, so the Intel slice of a universal build fails. Do not restore `universal-apple-darwin` until an Intel-compatible runtime is available and tested.
- Linux release packaging installs GTK 3, WebKitGTK 4.1, AppIndicator, librsvg, ALSA, and `patchelf` system development packages before building Tauri. These packages are build-runner dependencies for compiling and bundling `.deb`, `.rpm`, and AppImage artifacts; they are not npm packages.
- Rodio's output stream and sink stay on a dedicated audio thread. Do not move the platform stream into shared Tauri state or capture it in a generic spawned thread: CoreAudio's callback is not `Send`, which broke macOS compilation. Commands communicate with the audio thread through channels.
- The tag job and `build-and-publish` job explicitly request `contents: write`. The publishing job needs this permission for `tauri-action` to create/update the GitHub Release and upload assets. If release creation still returns `Resource not accessible by integration`, check repository/organization Actions settings for `GITHUB_TOKEN` write access.

### Terminal commands used for build troubleshooting

The fixes were workflow/configuration changes, so dependency changes on contributor machines were not required. The equivalent validation commands are:

```bash
# Frontend install/build
npm install --global npm@11.5.2 # CI pins npm to the lockfile-compatible version
npm ci
npm run lint
npm run test:coverage
npm run build

# Rust checks from src-tauri/
cargo fmt -- --check
cargo clippy -- -D warnings
cargo llvm-cov --all-features --workspace

# Desktop release build (invoke from repository root)
npm run tauri:build -- --target aarch64-apple-darwin # macOS Apple Silicon only
```

The Linux runner installs its build dependencies with:

```bash
sudo apt-get update
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev libasound2-dev patchelf
```

These installation commands run in GitHub Actions, not on contributors' machines. `npm install --global npm@11.5.2` pins the package manager only in the ephemeral runner; `npm ci` installs the versions already locked by the repository. Local Linux Tauri builds need the equivalent system packages for the developer's distribution; installed app packages may also depend on runtime WebKitGTK libraries. The macOS target requires an Apple Silicon host/toolchain and creates no Intel installer.

Local Husky hooks reinforce the same standards before a commit: lint-staged formats staged Angular files with Prettier and fixes TypeScript with ESLint; the Rust pre-commit checks run `cargo fmt --check` and `cargo clippy -- -D warnings`. Commitlint rejects commit messages that do not follow Conventional Commits.

## Continuous Delivery (CD)

After a commit is merged or pushed to `main`, the release workflow examines the Git history since the previous release. When the history contains a release-worthy Conventional Commit, it creates the next version tag, generates release notes, builds the Tauri application for Windows, macOS, and Linux, and publishes the binaries and notes as a GitHub Release. The workflow serializes release runs to avoid competing tag operations.

No release-worthy commit means no new version tag or release. Maintenance commits are added to the notes for a release when one is being created, but do not cause a release on their own.

## Conventional Commits to SemVer

The commit type and any breaking-change marker determine the version increment:

| Commit pattern                              | Version change                                                                       | Example           |
| ------------------------------------------- | ------------------------------------------------------------------------------------ | ----------------- |
| `fix(scope): ...`                           | Patch                                                                                | `1.0.0` → `1.0.1` |
| `feat(scope): ...`                          | Minor                                                                                | `1.0.1` → `1.1.0` |
| `BREAKING CHANGE: ...` in the commit footer | Major                                                                                | `1.1.0` → `2.0.0` |
| `chore:`, `docs:`, `refactor:`, `test:`     | No version bump by themselves; included in release notes when a release is published | —                 |

A breaking-change footer takes precedence over the ordinary `fix` or `feat` increment. Keep the scope and summary concise, and place `BREAKING CHANGE:` in the commit message footer so the release action can detect it.

## Workflow Files and Local Commands

- `.github/workflows/ci.yml` defines the pull-request and main-branch quality gates.
- `.github/workflows/release.yml` calculates tags and notes, then builds and publishes platform installers.
- `npm run test:coverage` runs Vitest with `@vitest/coverage-v8`.
- From `src-tauri`, `cargo llvm-cov --all-features --workspace` runs Rust tests and produces coverage output.

When changing a workflow, update this document and the contributor-facing `CI/CD & Release Flow` section in `README.md` in the same change.
