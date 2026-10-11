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
3. **Regra Estrita de Tipagem Angular:** É ESTRITAMENTE PROIBIDO declarar `interface` ou `type` diretamente dentro de ficheiros `.component.ts`, `.service.ts`, `.state.ts` ou `.directive.ts`. Todas as definições de tipagem devem ser isoladas em ficheiros independentes (como `*.model.ts` ou `*.types.ts`) e importadas. O objetivo é manter os ficheiros de lógica 100% focados no comportamento e garantir a separação de responsabilidades.
4. **State Management:** Use Angular Signals nativos para estado reativo síncrono e controle de UI.
5. **Styling & Dark Mode:** A UI deve ser minimalista e focada em estudos. O Dark Mode é gerenciado através da classe `.dark` do Tailwind. Use as paletas nativas do Tailwind com foco em tons Slate/Zinc para reduzir a fadiga ocular.
6. **I18n:** Nenhum texto *hardcoded* deve existir no HTML. Tudo deve ser envolvido no pipe ou diretiva do Transloco (`'KEY' | transloco`).
