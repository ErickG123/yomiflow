# Visual SRS: Image Occlusion

## Fluxo de dados

O editor Angular recebe arquivos PNG, JPEG ou WebP por upload ou colagem. O SVG de máscaras ocupa a mesma caixa de proporção da imagem e converte eventos de ponteiro em coordenadas normalizadas entre `0` e `1`: `x`/`y` identificam o início do retângulo e `width`/`height` o seu tamanho relativo. Assim, as máscaras permanecem alinhadas quando a imagem muda de tamanho.

Ao salvar, `save_image_occlusion_card` valida até 100 retângulos, grava os bytes da imagem em `<app_data_dir>/image_occlusion/<uuid>.<ext>` e serializa a lista de coordenadas em JSON na tabela SQLite `image_occlusion_cards`. O JSON guarda `x`, `y`, `width` e `height` como proporções, não pixels.

| Coluna | Tipo | Descrição |
| --- | --- | --- |
| `id` | `TEXT PRIMARY KEY` | UUID do cartão visual |
| `deck_id` | `TEXT` | Baralho relacionado, padrão `default`, removido em cascata com o baralho |
| `image_path` | `TEXT` | Caminho do arquivo local em `app_data_dir` |
| `masks_json` | `TEXT` | JSON de uma lista de retângulos normalizados |
| `created_at` | `TEXT` | Data/hora local de criação |
| `due_date`, `stability`, `difficulty`, `reps`, `state`, `last_review` | valores numéricos | Estado do agendador FSRS usado na revisão visual |

`list_image_occlusion_cards` devolve apenas metadados dos cartões novos ou vencidos, incluindo a quantidade de máscaras. Quando um cartão é selecionado, `get_image_occlusion_card` lê apenas o arquivo escolhido e devolve a imagem como data URL ao Angular, evitando carregar a coleção inteira na memória. Na revisão, cada retângulo ainda não respondido é desenhado opaco no SVG. O clique ou teclado revela apenas a máscara selecionada. Depois que todas são reveladas, a avaliação Again/Hard/Good/Easy chama `answer_image_occlusion_card`, que usa `calculate_fsrs_step` e registra a avaliação em `image_occlusion_review_logs`.

## Limites e formatos

O backend aceita PNG, JPEG e WebP até 15 MB e exige ao menos uma máscara com largura/altura positivas. O frontend e backend compartilham coordenadas normalizadas; o renderer usa um `viewBox` de 1000 × 1000 com a mesma proporção da imagem.
