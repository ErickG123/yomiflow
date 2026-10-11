# Pomodoro Global e Immersion Tracker

## Persistência SQLite

A tabela `study_sessions` armazena apenas blocos de foco concluídos com sucesso:

| Campo | Tipo | Regra | Uso |
| --- | --- | --- | --- |
| `id` | `TEXT` | Chave primária | UUID gerado pelo comando Rust |
| `category` | `TEXT` | Obrigatório | `active_reading`, `passive_listening` ou `srs_review` |
| `duration_minutes` | `INTEGER` | Obrigatório, maior que zero | Duração configurada do bloco concluído |
| `date` | `TEXT` | Obrigatório, horário local por padrão | Data/hora local de conclusão (`YYYY-MM-DD HH:MM:SS`) |

`save_study_session` valida categoria e duração (1–180 minutos) antes de inserir a sessão na conexão SQLite compartilhada. Intervalos, ciclos cancelados e blocos interrompidos não são contabilizados.

## Estado Angular

O `PomodoroWidget` é montado no rodapé da Sidebar e permanece ativo durante a navegação. Signals armazenam `status` (`idle`, `running`, `paused`), `phase` (`focus`, `break`), duração de foco e intervalo, categoria e segundos restantes. O prazo absoluto (`Date.now()`) permite recompor o tempo decorrido após atrasos do event loop quando a janela está minimizada. Ao terminar o foco, o componente persiste a sessão por IPC e envia notificação nativa via `tauri-plugin-notification`; ao terminar o intervalo, envia notificação e prepara o próximo bloco de foco.

As preferências disponíveis são foco de 25 ou 50 minutos, intervalo de 5 ou 10 minutos e as três categorias listadas acima. Os rótulos e mensagens são traduzidos via Transloco.
