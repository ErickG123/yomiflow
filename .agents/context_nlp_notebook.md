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
  - `models/notebook.model.ts`: Todas as tipagens (`MorphologicalToken`, `NotebookEntry`, `NotebookState`).
  - `services/notebook.service.ts`: Comunicação IPC via `@tauri-apps/api/core` (`analyze_japanese_text`, `save_notebook_entry`, `get_notebook_entries`) e parser HTML enriquecido.
  - `components/notebook-editor/`: Editor TipTap montado em container nativo, suporte a Markdown puro e painel morfológico split-screen reativo com Angular Signals.

## 5. Comandos de Validação e Testes
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
