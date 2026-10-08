# YomiFlow

> **YomiFlow** é uma aplicação desktop offline de alto desempenho projetada para estudantes do idioma japonês, combinando um dicionário de busca instantânea com um Sistema de Repetição Espaçada (SRS) integrado e focado em produtividade minimalista.

---

## ✨ Features Atuais (MVP)

- ⚡ **Busca Offline de Alta Velocidade:** Consulta rápida e reativa indexada em SQLite local com correspondência de Kanji, Kana (Reading) e significados.
- 📂 **Importador de Dicionários Yomichan:** Suporte a importação em lote de diretórios contendo bancos de termos (`term_bank_*.json`) via transações atômicas no SQLite.
- 🃏 **Integração com SRS:** Adicione termos diretamente da pesquisa de dicionário para a sua fila de repetição espaçada com um único clique.
- 🧘 **Modo Zen de Revisão:** Interface minimalista, sem distrações, focada na memorização ativa com suporte completo a atalhos de teclado (`Espaço` para revelar e `1` a `4` para classificar).
- 🌙 **Dark Mode Nativo:** Paleta de tons escuros (Zinc) pensada para reduzir a fadiga ocular durante longas sessões de estudo.

---

## 🛠️ Tech Stack

- **Frontend:** [Angular 22](https://angular.dev/) (Standalone Components, Signals, Router, SCSS)
- **Estilização & Ícones:** [Tailwind CSS](https://tailwindcss.com/) + [PrimeIcons](https://primeng.org/icons)
- **Internacionalização (i18n):** [@ngneat/transloco](https://ngneat.github.io/transloco/)
- **Testes & Qualidade:** [Vitest](https://vitest.dev/), Testing Library, [ESLint](https://eslint.org/) & [Prettier](https://prettier.io/)
- **Desktop Runtime:** [Tauri v2](https://v2.tauri.app/)
- **Backend Core:** [Rust](https://www.rust-lang.org/)
- **Banco de Dados Local:** [SQLite](https://sqlite.org/) via crate `rusqlite` (bundled)

---

## 📋 Pré-requisitos

Antes de iniciar, certifique-se de possuir instalado na sua máquina:

1. **Node.js** (v20+ recomendado) e **npm**
2. **Rust** e **Cargo** (via [rustup](https://rustup.rs/))
3. **Pré-requisitos do SO para o Tauri v2:**
   - **Windows:** Microsoft Visual Studio C++ Build Tools & WebView2 Runtime (geralmente pré-instalado no Windows 10/11).
   - **Linux:** Pacotes de desenvolvimento WebKit2GTK, GTK3 e dependências do sistema:
     ```bash
     sudo apt-get update && sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf
     ```
   - **macOS:** Xcode Command Line Tools (`xcode-select --install`).

---

## 🚀 Como Executar Localmente

1. **Clone o repositório:**
   ```bash
   git clone https://github.com/seu-usuario/yomiflow.git
   cd yomiflow
   ```

2. **Instale as dependências do Frontend:**
   ```bash
   npm install
   ```

3. **Inicie o ambiente de desenvolvimento Tauri:**
   ```bash
   npm run tauri dev
   ```
   > Esse comando compilará o core em Rust, inicializará o servidor de desenvolvimento do Angular e abrirá a janela desktop nativa.

4. **Comandos Úteis:**
   - Testes unitários (Frontend): `npm run test`
   - Linting de código: `npm run lint`
   - Build do Frontend: `npm run build`
   - Checagem do Backend: `cd src-tauri && cargo check`
   - **Geração de Ícones Nativos:** Coloque uma imagem `app-icon.png` (1024x1024) na raiz e execute:
     ```bash
     npm run tauri:icon
     ```
   - **Build de Produção (Instalador Nativo Local):** Para gerar o instalador `.exe` (Windows) ou `.deb`/`.AppImage` (Linux):
     ```bash
     npm run tauri:build
     ```
     > Os instaladores finais e binários otimizados serão gerados na pasta `src-tauri/target/release/bundle/`.

---

## 📁 Estrutura do Projeto

O projeto segue uma arquitetura baseada em domínios (*Feature-Based Architecture*) no cliente e microsserviço IPC no backend:

```text
yomiflow/
├── .github/workflows/       # Pipelines de CI/CD (GitHub Actions)
├── agents/                  # Diretrizes e regras arquiteturais do projeto
├── public/                  # Arquivos estáticos públicos e traduções i18n
├── src/                     # Aplicação Frontend (Angular 22)
│   ├── app/
│   │   ├── core/            # Serviços singleton, loaders e interceptors globais
│   │   ├── shared/          # Componentes reutilizáveis, pipes e diretivas
│   │   └── features/        # Módulos de domínio de negócio
│   │       ├── dictionary/  # Busca no dicionário e integração de importação
│   │       └── srs/         # Sessão de revisão (Zen Mode) e agendamento
│   ├── styles.scss          # Estilos globais e diretivas do Tailwind
│   └── test-setup.ts        # Configuração do ambiente de testes Vitest
├── src-tauri/               # Backend nativo Desktop (Rust + Tauri)
│   ├── Cargo.toml           # Dependências Rust (rusqlite, uuid, tauri-plugins)
│   ├── tauri.conf.json      # Configuração de janelas, build e permissões
│   └── src/
│       ├── main.rs          # Ponto de entrada do executável Tauri
│       ├── lib.rs           # Registro de plugins e invoke handlers IPC
│       ├── db.rs            # Migrações SQLite e inicialização do banco local
│       ├── models.rs        # Estruturas de dados compartilhadas
│       └── commands.rs      # Comandos IPC (search, add_to_srs, import, get_due_cards)
├── tailwind.config.js       # Configuração do Tailwind CSS
├── vitest.config.ts         # Configuração de execução de testes unitários
└── package.json             # Dependências e scripts do ecossistema Node.js
```

---

## 📄 Licença

Distribuído sob a licença MIT. Consulte `LICENSE` para mais informações.
