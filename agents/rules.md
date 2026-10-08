# System Rules & Guidelines (Angular + Tauri Project)

## Stack & Tech
- Framework: Angular 22 (Standalone Components).
- Styling: SCSS + TailwindCSS.
- Icons: PrimeIcons.
- i18n: Transloco.
- Testing: Vitest.
- Formatting: Prettier + ESLint.

## Architectural Rules
1. **Feature-Based Architecture:** Todo o código de negócio deve viver dentro de `src/app/features/`. Use o `core/` apenas para singletons globais (como tokens de autenticação ou setup de banco) e `shared/` para UI components genéricos (botões, inputs).
2. **File Separation:** É ESTRITAMENTE PROIBIDO usar `template` ou `styles` inline no `@Component`. Sempre use arquivos `.html` e `.scss` separados.
3. **Interfaces & Models:** É ESTRITAMENTE PROIBIDO declarar `interface`, `type` ou `enum` dentro de arquivos `.component.ts` ou `.service.ts`. Toda tipagem deve viver em um arquivo `.interface.ts` ou `.model.ts` próprio. Nunca coloque interfaces no mesmo arquivo de componentes lógicos.
4. **State Management:** Use Angular Signals nativos para estado reativo síncrono e controle de UI.
5. **Styling & Dark Mode:** A UI deve ser minimalista e focada em estudos. O Dark Mode é gerenciado através da classe `.dark` do Tailwind. Use as paletas nativas do Tailwind com foco em tons Slate/Zinc para reduzir a fadiga ocular.
6. **I18n:** Nenhum texto *hardcoded* deve existir no HTML. Tudo deve ser envolvido no pipe ou diretiva do Transloco (`'KEY' | transloco`).

## Goal (Phase 1)
Construir o MVP do **YomiFlow**, um Dicionário de Japonês offline integrado a um sistema de Repetição Espaçada (SRS), visando altíssima performance. O Angular atua como um cliente minimalista de alta velocidade, e o Tauri (Rust) como micro-backend local lidando com o SQLite e File System.
