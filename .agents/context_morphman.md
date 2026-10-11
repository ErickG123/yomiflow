# Morphological Readability Analyzer (Offline MorphMan)

## Fluxo

O comando Tauri `analyze_text_readability(text)` usa o segmentador Lindera/IPADIC já inicializado pelo domínio NLP. Cada token lexical preserva a superfície e os offsets no texto original, e extrai o lemma (campo de detalhes IPADIC) e a leitura normalizada em hiragana. Pontuação e símbolos são excluídos da contagem. Os offsets retornados ao frontend são convertidos para unidades UTF-16 para que `String.slice` no JavaScript destaque exatamente a ocorrência original, inclusive em textos que contenham caracteres suplementares.

Para cada par único `(lemma, reading)`, o comando consulta `dictionary_entries` associado a `cards` por `entry_id`, relacionando `kanji` ao lemma e `reading` à leitura. Um `LEFT JOIN` com `review_logs` obtém a revisão mais recente. A consulta preparada e o cache em memória evitam uma consulta repetida para cada ocorrência do mesmo termo.

## Cobertura e retenção

`total_words` conta as ocorrências lexicais do texto, inclusive repetições. `known_words` conta as ocorrências cujo lemma/leitura tem ao menos um cartão em qualquer baralho. `comprehension_percentage` é `known_words / total_words * 100`, arredondado para uma casa decimal. O array `unknown_words` contém cada ocorrência sem cartão, com superfície, lemma e offsets para o destaque no frontend.

A retenção não muda a definição de cobertura: ela é apresentada separadamente como `average_retention_percentage` entre os termos com cartão. Por cartão, usa-se a retrievability FSRS derivada de estabilidade e dias desde a revisão mais recente; cartões de aprendizagem têm fator de maturidade reduzido, e o melhor valor entre cartões duplicados representa a palavra. Um cartão novo existe como vocabulário conhecido para cobertura, ainda que sua retenção atual seja zero. Isso permite ao Cenário I verificar que adicionar um cartão aumenta matematicamente a compreensão estimada.

## Estado da interface

A tela `/readability` mantém o texto, carregamento, erro e resultado em Signals Angular. Após a resposta IPC, os offsets da lista `unknown_words` segmentam o texto; somente segmentos fornecidos pelo backend são renderizados como desconhecidos. O gráfico circular representa a percentagem de cobertura e a retenção média fica como métrica complementar. Todas as strings visíveis usam Transloco em PT, EN e ES.
