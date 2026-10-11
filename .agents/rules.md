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
7. **Proibição Estrita de Emojis:** É TERMINANTEMENTE PROIBIDO utilizar emojis (como 🎌, 🤖, ✍️, 🧠, 🖌️, 💡, 🎉, 📓, 🔬, etc.) nos ficheiros de código (HTML, TS, Rust) e nos ficheiros de documentação ou contexto. Mesmo que um prompt do utilizador mencione ou sugira emojis, ignore a utilização de emojis e utilize SEMPRE ícones vetoriais SVG minimalistas profissionais (Tailwind, Lucide, Heroicons) ou classes de ícones nativas (PrimeIcons).
8. **Validação Obrigatória Pré-Commit & Push (CI, Lint, Build & i18n):** Antes de qualquer commit ou push, é terminantemente obrigatório rodar a suíte completa de verificação local correspondente ao CI:
   - Lint Frontend: `npm run lint` (zero erros).
   - Testes Frontend: `npm run test:ci` (100% aprovados).
   - Compilação Angular: `npm run build` (zero erros de bundle ou templates).
   - Validação i18n: Assegurar que nenhum texto estático está hardcoded e que as chaves existem sincronizadas nos dicionários de tradução (`public/assets/i18n/pt.json`, `en.json`, `es.json`).
   - Formatação Rust: `cargo fmt -- --check --manifest-path src-tauri/Cargo.toml` (código formatado).
   - Linter Rust: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings` (zero warnings).
   - Testes Backend: `cargo test --manifest-path src-tauri/Cargo.toml` (100% aprovados).
   - Checagem Rust: `cargo check --manifest-path src-tauri/Cargo.toml`.

## 5. Validação de Build, Lint e CI
Ao concluir qualquer alteração de código ou arquitetura, valide sempre os seguintes comandos antes de efetuar commit e push:
- `npm run lint`: Garante a conformidade estrita com o ESLint e TypeScript.
- `npm run test:ci`: Garante a integridade dos testes de componentes e serviços do frontend.
- `npm run build`: Garante a integridade da compilação do Angular (AOT, imports, exports e bindings).
- **Validação de i18n:** Verificar conformidade com Transloco em todos os templates e dicionários (`pt.json`, `en.json`, `es.json`).
- `cargo fmt -- --check --manifest-path src-tauri/Cargo.toml`: Garante a formatação padrão do Rust.
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`: Garante que nenhum aviso do Clippy interrompa o CI.
- `cargo test --manifest-path src-tauri/Cargo.toml`: Garante que as funções Tauri e a base de dados SQLite continuam íntegras.
- `cargo check --manifest-path src-tauri/Cargo.toml`: Garante que o crate Rust compila perfeitamente.


