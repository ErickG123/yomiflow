# Contexto de Arquitetura: Caderno Híbrido (NLP & TipTap)

## 1. Visão Geral
O **Caderno Híbrido** (`src/app/features/notebook/`) é um editor Markdown voltado para imersão no aprendizado do japonês com análise morfológica em tempo real powered by **Lindera** (Rust backend com dicionário IPADIC embutido).

## 2. Estrutura de Tabelas SQLite
Definida em `src-tauri/src/db/setup.rs`:
```sql
CREATE TABLE IF NOT EXISTS notebook_entries (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    content_md TEXT NOT NULL,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);
```

### Queries de Persistência (`src-tauri/src/commands/nlp.rs`)
- **Upsert / Salvar Nota:**
  ```sql
  INSERT INTO notebook_entries (id, title, content_md, created_at, updated_at)
  VALUES (?1, ?2, ?3, ?4, ?5)
  ON CONFLICT(id) DO UPDATE SET
      title=excluded.title,
      content_md=excluded.content_md,
      updated_at=excluded.updated_at;
  ```
- **Listar Notas Recentes:**
  ```sql
  SELECT id, title, content_md, created_at, updated_at
  FROM notebook_entries
  ORDER BY updated_at DESC;
  ```

## 3. Mapeamento de Classes Gramaticais (POS Mapping)
O backend (`lindera`) categoriza cada morfema de acordo com o dicionário IPADIC e categoriza simplificadamente para o frontend:

| Categoria Rust (`pos`) | Tags IPADIC | Classe Visual Tailwind | Descrição |
| :--- | :--- | :--- | :--- |
| `verb` | 動詞 | `text-red-400 font-semibold` | Verbos (ex: 食べる, 行く, 走る) |
| `particle` | 助詞 | `text-blue-400 font-semibold` | Partículas gramaticais (ex: は, が, を, に, で) |
| `noun` | 名詞 | `text-green-400 font-semibold` | Substantivos (ex: 私, 学生, 日本語) |
| `adjective` | 形容詞 | `text-amber-400` | Adjetivos do tipo I/Na |
| `auxiliary` | 助動詞 | `text-purple-400` | Cópula e verbos auxiliares (ex: です, ます, だ) |
| `other` | Outros | `text-slate-300` | Pontuação, símbolos, conjunções |

## 4. Frontend: TipTap & Feature Isolation
- Localização: `src/app/features/notebook/`
- Arquitetura estrita feature-based:
  - `models/notebook.model.ts`: Todas as tipagens (`MorphologicalToken`, `NotebookEntry`, `AnalysisMode`).
  - `services/notebook.service.ts`: Comunicação IPC via `@tauri-apps/api/core` (`analyze_japanese_text`, `save_notebook_entry`, `get_notebook_entries`) e parser HTML enriquecido.
  - `components/notebook-editor/`: Editor TipTap montado em container nativo, suporte a Markdown puro e painel morfológico split-screen reativo com Angular Signals.

## 5. Modos de Análise Gramatical & Estratégia de Debounce
- **Modo Automático (`auto`):**
  - O evento `onUpdate` do TipTap alimenta um fluxo RxJS `Subject<string>` com operador `debounceTime(750)`.
  - Evita sobrecarga de chamadas IPC ao Rust e garante que a análise morfológica só dispara 750ms após o utilizador cessar a digitação.
- **Modo Manual (`manual` - Padrão):**
  - O evento `onUpdate` apenas sincroniza a string Markdown sem invocar o analisador Lindera.
  - O utilizador clica no botão com ícone de varinha/faíscas **"Analisar Gramática"**, que analisa o texto atual sob demanda.

## 6. Validação de Build, Lint & CI
Ao concluir qualquer implementação referente ao caderno híbrido ou extensões TipTap, valide estritamente antes de qualquer commit ou push:
```bash
npm run lint
npm run test:ci
npm run build
cargo fmt -- --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```
*Certifique-se também da conformidade de i18n (Transloco) e de que todas as strings de UI estão devidamente internacionalizadas nos dicionários de tradução.*

- **Preservação de Posição do Cursor no TipTap:**
  - Desafio: Injetar HTML colorido via `setContent()` costuma resetar o cursor para o final do documento.
  - Solução:
    1. Salva-se a seleção do ProseMirror antes do update: `const { from, to } = this.editor.state.selection;`.
    2. Invoca-se `editor.commands.setContent(wrappedHtml, { emitUpdate: false })` com a flag `emitUpdate: false` para evitar ciclos infinitos no listener de update.
    3. Restaura-se a seleção clampada ao novo tamanho do documento: `editor.commands.setTextSelection({ from: safeFrom, to: safeTo })`.

## 6. Arquitetura de Domínio (DDD) e Segurança (SecOps)
- **Isolamento de Domínio Puro (`src-tauri/src/domain/nlp.rs`):**
  - A lógica pura de segmentação morfológica Lindera e classificação gramatical (`segment_and_tag_japanese`, `map_pos_category`) está desacoplada da camada Tauri.
  - O comando `analyze_japanese_text` em `src-tauri/src/commands/nlp.rs` atua estritamente como adaptador de aplicação.
- **Prevenção de DoS por Exaustão de Recursos (IPC Limit):**
  - Constante de proteção: `MAX_JAPANESE_TEXT_BYTES = 50_000` bytes (~50 KB de texto em UTF-8).
  - Previne exaustão de memória ou bloqueio prolongado da CPU se o utilizador colar textos de proporções astronômicas no editor.
- **Prevenção de XSS (Cross-Site Scripting):**
  - O pipe `SafeHtmlPipe` (`src/app/shared/pipes/safe-html.pipe.ts`) realiza sanitização estrita via `DomSanitizer.sanitize(SecurityContext.HTML, ...)`.
  - Tags `<script>` e manipuladores de eventos (`onerror`, `onload`, `onclick`) são neutralizados antes de qualquer injeção no DOM do TipTap ou da interface.

## 7. Comandos de Validação e Testes
- **Backend (Rust):**
  ```bash
  cargo test commands::nlp
  cargo test --lib
  ```
- **Frontend (Angular):**
  ```bash
  npm test -- --include src/app/features/notebook/**/*.spec.ts
  npm run build
  ```

