# YomiFlow - Japanese Language Immersion & Acquisition Studio

[![Tauri v2](https://img.shields.io/badge/Tauri-v2.12-24C8D8?style=flat-square&logo=tauri&logoColor=white)](https://v2.tauri.app/)
[![Rust](https://img.shields.io/badge/Rust-2024%20Edition-DEA584?style=flat-square&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Angular](https://img.shields.io/badge/Angular-22%20Signals-DD0031?style=flat-square&logo=angular&logoColor=white)](https://angular.dev/)
[![SQLite](https://img.shields.io/badge/SQLite-WAL%20Mode-003B57?style=flat-square&logo=sqlite&logoColor=white)](https://www.sqlite.org/)
[![Offline First](https://img.shields.io/badge/Offline--First-100%25-10B981?style=flat-square&logo=speedtest&logoColor=white)](https://github.com/)

> **YomiFlow** é uma aplicação desktop *offline-first* ultrarrápida desenhada para unificar todo o ecossistema de aquisição e imersão no idioma japonês: desde a leitura e mineração de vocabulário em mangás e vídeos até à retenção na memória por SRS e prática ativa de caligrafia vetorial.

---

## 📸 Visão Geral da Interface

| Dashboard & Heatmap | Leitor de Mangás & OCR | Prática de Caligrafia (Kanji Pad) |
| :---: | :---: | :---: |
| ![Dashboard Screenshot](https://raw.githubusercontent.com/yomiflow/assets/main/screenshots/dashboard.png) | ![Manga Reader Screenshot](https://raw.githubusercontent.com/yomiflow/assets/main/screenshots/reader.png) | ![Kanji Pad Screenshot](https://raw.githubusercontent.com/yomiflow/assets/main/screenshots/kanjipad.png) |

---

## 🌟 Destaques e Funcionalidades

### 1. Dicionário & SRS (O Núcleo de Aquisição)
- **Pesquisa Instantânea Yomichan:** Busca indexada em SQLite local por Kanji, Leitura em Hiragana/Katakana e Significados, com suporte a decomposição detalhada de radicais.
- **Importação Massiva de Dados:** Ingestão atômica de bancos de termos Yomichan (`term_bank_*.json`), bancos de Kanjis (`kanji_bank_*.json`) e bases JLPT (N5 a N1).
- **Algoritmo SRS (Repetição Espaçada FSRS):** Agendamento inteligente de revisões baseado nos feedbacks *Again (1)*, *Hard (2)*, *Good (3)* e *Easy (4)*, persistido diretamente no SQLite.

### 2. Gamificação & Métricas de Estudo
- **Heatmap de Contribuições:** Visualização visual no estilo GitHub das revisões dos últimos 112 dias (16 semanas) com 5 níveis de intensidade cromática.
- **Contador de Ofensivas (Streaks):** Cálculo de ofensiva atual (*Current Streak*) e recorde histórico (*Max Streak*), mantendo a motivação diária.

### 3. Leitor de Mangás & OCR (Consumo Estático)
- **Suporte Multiformato em Memória:** Leitura direta de ficheiros `.cbz`, `.zip`, `.cbr` e `.rar` sem necessidade de descompactar pastas inteiras no disco.
- **Motor de Recorte Cirúrgico & OCR:** Seleção livre com retângulo interativo sobre balões de fala, com pré-processamento de padding branco e conversão Luma8 para OCR preciso sem alucinações de bordas.
- **Upgrade de IA Local (MangaOCR com ONNX):** Pipeline opcional de visão-texto executado em ONNX Runtime (`ort`) 100% local e offline.

### 4. Prática Ativa (Audição e Escrita)
- **Áudio Nativo (TTS Offline):** Reprodução instantânea de pronúncia em japonês nativo (`ja-JP`) integrada ao dicionário e às sessões de estudo via atalhos de teclado.
- **Kanji Pad Vetorial (KanjiVG):** Renderização vetorial dos traços oficiais do KanjiVG com canvas de caligrafia interativo e motor matemático de avaliação geométrica de escrita.

### 5. Consumo Dinâmico de Vídeo (Fase 4)
- **Player de Vídeo com Hover Dictionary:** Reprodução de arquivos `.mkv` e `.mp4` via streaming local do Tauri, com extração de legendas embutidas via FFmpeg e pausa instantânea ao passar o cursor sobre as palavras.
- **Corte de Áudio Nativo para SRS:** Recorte automatizado do intervalo exato da legenda em arquivo `.mp3` via FFmpeg, criando flashcards multimídia completos.

### 6. Ferramentas de Alta Imersão & Nuvem (Fase 5)
- **Dicionário de Frequência:** Badges visuais de classificação de palavras (*Core Top 1.5K*, *Comum Top 5K*, *Rara*) via `LEFT JOIN` indexado no SQLite e ingestão massiva atômica de corpus (ex: Innocent Corpus).
- **Pitch Accent Visual:** Curvatura de acento tonal com suporte a padrões binários (`[0, 1, 1, 0]`), strings HL e índices de queda (*drop*), integrado a botão de áudio nativo TTS.
- **Decomposição de Kanjis & Mnemónicas (WaniKani Style):** Visualização de radicais e histórico de mnemónicas do utilizador com auto-save invisível (`(blur)`) no SQLite.
- **Cloud Sync (Bring Your Own Storage - Google Drive):** Sincronização e backup direto do banco `yomiflow.db` no Google Drive via OAuth2 nativo (utilizando as crates `oauth2`, `tiny_http` e `reqwest`). O utilizador mantém **100% da propriedade e privacidade dos seus dados**, sem necessidade de servidores intermediários de terceiros.

---

## 🛠️ Stack Tecnológico & Filosofia de Arquitetura

- **Frontend:** [Angular 22](https://angular.dev/) (Standalone Components, Signals reativos, OnPush Change Detection, Tailwind CSS Zinc Palette, i18n via `@ngneat/transloco`).
- **Desktop Runtime:** [Tauri v2](https://v2.tauri.app/) (Pegada de memória mínima, isolamento seguro entre processos).
- **Backend Core:** [Rust](https://www.rust-lang.org/) (2024 Edition, zero alocações desnecessárias, IPC tipado com `oauth2`, `tiny_http`, `reqwest`, `rusqlite`, `ort`).
- **Banco de Dados Local:** [SQLite](https://sqlite.org/) via crate `rusqlite` com modos de alta performance (`PRAGMA journal_mode = WAL`, `PRAGMA synchronous = NORMAL`, `PRAGMA temp_store = MEMORY`).

### Filosofia Arquitetural
1. **Domain-Driven Design (DDD):** Separação estrita entre o Domínio puro (cálculos matemáticos de SRS, validação de coordenadas, modelos de dados), a Aplicação (comandos e eventos Tauri) e a Infraestrutura (SQLite, FFmpeg, IO de arquivos).
2. **Princípio DRY (Don't Repeat Yourself):** Reutilização de utilitários puros de texto, higienização de inputs e tipos desacoplados.
3. **Segurança Rigorosa:**
   - Prevenção contra *SQL Injection* utilizando exclusivamente prepared statements parametrizados.
   - Proteção contra *Path Traversal* com validação e sanitização estrita de nomes de arquivos e caminhos no backend Rust.
   - Proteção contra *XSS* sanitizando conteúdo dinâmico através do `DomSanitizer` (`SecurityContext.HTML`).

---

## 🚀 Instalação e Desenvolvimento

### Pré-requisitos
1. **Node.js** (v20+ recomendado) e **npm**
2. **Rust & Cargo** (v1.90+ via [rustup.rs](https://rustup.rs/))
3. **FFmpeg** (opcional, para extração de legendas e cortes de áudio em vídeos `.mkv`)

### Passo a Passo

1. **Clonar o Repositório:**
   ```bash
   git clone https://github.com/seu-usuario/yomiflow.git
   cd yomiflow
   ```

2. **Instalar Dependências:**
   ```bash
   npm install
   ```

3. **Executar em Modo de Desenvolvimento:**
   ```bash
   npm run tauri dev
   ```

---

## 🧪 Qualidade e Testes Automatizados

O projeto conta com pipelines de testes unitários e de integração em ambas as camadas:

```bash
# Executar a suíte de testes do Backend Rust
cargo test --lib --manifest-path src-tauri/Cargo.toml

# Executar a verificação estrita do linter Rust (sem avisos)
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings

# Executar a suíte de testes do Frontend Angular (Vitest)
npm run test:ci

# Executar o linter TypeScript/Angular
npm run lint

# Compilar o bundle de produção
npm run build
```

---

## 📄 Licença

Distribuído sob a licença MIT. Consulte `LICENSE` para mais detalhes.
