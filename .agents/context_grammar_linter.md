# Grammar Linter for TipTap

## Analysis flow

The Tauri IPC command `lint_japanese_text` delegates to the pure Lindera domain function. Lindera token surfaces and lemmas are inspected for a particle `を` followed by a recognized potential verb, including the lemma for `分かる` and forms such as `読める`. The first heuristic recommends `が` and returns the UTF-16 code-unit range of the particle, its original text, replacement, and explanation.

Example input: `
日本語を分かります
`. Expected correction: `
日本語が分かります
`. An emoji before the sentence is covered by a test to ensure UTF-16 positions remain aligned with ProseMirror.

## TipTap integration and responsiveness

`GrammarLinterExtension` is a ProseMirror plugin. It receives backend suggestions after a 500 ms debounce, maps the UTF-16 offsets into textblock positions, and adds inline decorations with an orange wavy underline. Decorations are cleared on document edits while the next debounced request runs.

Hover or click on a decorated span opens a small Angular tooltip with the explanation and a replacement button. The tooltip applies the suggestion only to the decorated range through `insertContentAt`. The note text is sent to Rust locally and no network service is used.

## Tests

Rust unit coverage checks the particle rule, the exact replacement range, the UTF-16 offset after an emoji, and the corrected sentence. The TipTap extension tests verify single- and multi-block position mapping and that the decoration is removed when the suggestion is applied.

```bash
cd src-tauri && cargo test grammar_linter_suggests_ga_for_wakaru_potential_usage
npm run test:ci -- --run src/app/features/notebook/extensions/grammar-linter.extension.spec.ts
```
