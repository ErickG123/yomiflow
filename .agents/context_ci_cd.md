# CI/CD and Automated Release Rules

> **CRITICAL DIRECTIVE:** Keep workflow behavior, commit conventions, and these release rules aligned. Do not create release tags manually; the GitHub Actions release workflow owns version tagging and publication.

## Continuous Integration (CI)

The CI workflow runs for pull requests targeting `main` and pushes to `main`. A failed quality gate blocks the workflow from succeeding. It checks Prettier formatting on changed frontend TypeScript, HTML, and SCSS files, runs ESLint, executes Vitest with the V8 coverage provider, checks Rust formatting and Clippy, runs Rust tests with `cargo-llvm-cov`, and builds the Angular production bundle. Both frontend and Rust coverage commands must complete and generate their reports; no numeric coverage threshold is currently configured.

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
