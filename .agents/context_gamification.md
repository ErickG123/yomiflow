# Contexto de Gamificação e Planeamento Temporal

## 1. Agregação Temporal no Backend (SQLite & Rust)
O motor de agregação temporal (`src-tauri/src/commands/stats.rs`) é responsável por calcular o histórico de ofensivas e projetar o mapa térmico dos próximos 30+ dias de revisões programadas pelo algoritmo SRS.

### Queries de Agrupamento por Data
- **Previsão Futura (Review Forecast):**
  Agrupa os cartões da tabela `cards` pelo campo `due_date` formatado em data local ISO (`YYYY-MM-DD`):
  ```sql
  SELECT 
      DATE(datetime(due_date, 'unixepoch', 'localtime')) as forecast_date, 
      COUNT(*) as count 
  FROM cards 
  WHERE DATE(datetime(due_date, 'unixepoch', 'localtime')) BETWEEN ?1 AND ?2
  GROUP BY forecast_date 
  ORDER BY forecast_date ASC;
  ```
- **Histórico de Estudo (Study History Tracker):**
  Agrupa os logs concluídos da tabela `review_logs` pelo campo `review_datetime`:
  ```sql
  SELECT 
      DATE(datetime(review_datetime, 'unixepoch', 'localtime')) as history_date, 
      COUNT(*) as count 
  FROM review_logs 
  WHERE strftime('%Y-%m', datetime(review_datetime, 'unixepoch', 'localtime')) = ?1
  GROUP BY history_date 
  ORDER BY history_date ASC;
  ```

## 2. Tratamento de Timezones (Local vs UTC)
- **Armazenamento no SQLite:** Todos os timestamps (`due_date`, `review_datetime`, `created_at`) são persistidos estritamente em **segundos epoch UTC** (`i64`), garantindo interoperabilidade entre diferentes fusos horários.
- **Transformação no SQLite:** A função SQL `datetime(col, 'unixepoch', 'localtime')` seguida de `DATE()` converte o timestamp universal para a data local do utilizador, evitando que revisões feitas próximas à meia-noite sofram translação de dia.
- **Normalização no Angular:** O componente `CalendarViewComponent` gera as chaves de comparação no formato local estrito `YYYY-MM-DD` (usando `getFullYear()`, `getMonth() + 1` e `getDate()`), alinhando 1:1 com a resposta do backend.

## 3. Escala Térmica e Variáveis Visuais (Heatmap)
As intensidades térmicas de estudo do calendário obedecem ao seguinte degradê semântico baseado em Tailwind / RGBA:

| Volume de Revisões Diárias | Cor de Fundo | Classe Tailwind Semântica | Descrição |
| :--- | :--- | :--- | :--- |
| `0` | `rgba(24, 24, 27, 0.6)` | `bg-zinc-900` | Sem atividade |
| `1 a 5` | `rgba(6, 78, 59, 0.4)` | `bg-emerald-950` | Ofensiva leve |
| `6 a 15` | `rgba(4, 120, 87, 0.5)` | `bg-emerald-700` | Atividade moderada |
| `16 a 30` | `rgba(16, 185, 129, 0.6)` | `bg-emerald-500` | Meta de estudos diária |
| `> 30` | `rgba(52, 211, 153, 0.8)` | `bg-emerald-400` | Imersão intensa / Alta produtividade |
| **Previsão Futura** | `rgba(244, 63, 94, 0.2)` | `bg-rose-500/20 text-rose-300` | Badges de cartões agendados |

## 4. Integração com o Caderno Híbrido
- Na visão mensal, dias que possuem anotações criadas ou editadas no caderno exibem um ponto pulsante âmbar (`.bg-amber-400`).
- Ao clicar em qualquer célula do calendário, o painel lateral exibe a lista completa de anotações vinculadas àquela data com link direto para o editor Markdown TipTap (`/notebook`).

## 5. Validação de Build, Lint & CI
Ao concluir qualquer implementação referente a este contexto, execute a validação completa de compilação, lint e testes antes de cada commit e push:
```bash
npm run lint
npm run test:ci
npm run build
cargo fmt -- --check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
```
*Garantir 100% de cobertura de i18n via Transloco em novos elementos visuais.*
