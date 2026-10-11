# Estudo Dinâmico e Fila Intercalada

## Construção e embaralhamento da fila

`get_interleaved_review_queue` consulta as fontes locais de estudo e reúne cartões vencidos ou novos de vocabulário, cartões de oclusão ligados a baralhos, notas de gramática ligadas a baralhos e até oito kana com `last_practiced` zerado ou anterior a 24 horas. O vocabulário e as notas/imagens são limitados a 100 itens por tipo para manter a resposta IPC controlada. Todos os baralhos existentes são considerados ativos; o comando não restringe pelo `deckId` da rota.

Rust aplica Fisher–Yates separadamente às quatro listas e então as intercala por tipo, avançando para o próximo tipo disponível a cada carta. Um estado xorshift sem dependência externa usa o relógio atual como semente, variando a ordem dentro de cada tipo e o tipo inicial entre chamadas. Quando há mais de um tipo disponível, não há duas cartas do mesmo tipo consecutivas; se uma lista acaba, as demais continuam.

## União discriminada de tipos

O modelo `InterleavedReviewCard` serializa um campo `cardType` com quatro variantes: `vocabulary`, `image_occlusion`, `grammar` e `kana`. A aplicação Angular espelha a união discriminada em `interleaved-card.model.ts`; o `@switch (card.cardType)` do `StudySessionComponent` renderiza uma UI específica para cada tipo e destrói a anterior ao trocar de tipo.

- Vocabulário conserva `answer_card` e FSRS.
- Oclusão carrega a imagem local somente quando a carta é selecionada e usa `answer_image_occlusion_card`.
- Gramática revela o conteúdo da nota e agenda sua próxima revisão por `answer_grammar_review`.
- Kana apresenta o caractere, valida romaji e atualiza `kana_mastery` por `update_kana_score`.

Depois de cada resposta, o frontend remove a carta atual do snapshot e limpa flip, máscaras reveladas e entrada de kana antes de mostrar a próxima. Uma troca de componente não herda estado de resposta da carta anterior.
