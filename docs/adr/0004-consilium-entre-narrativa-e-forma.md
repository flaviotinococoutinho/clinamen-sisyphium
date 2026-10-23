# ADR 0004 — Consilium entre a situação narrativa e a forma

- **Status:** aceito
- **Relacionados:** `06-consilium.md`; `00-metacognicao.md` §6; REQ-C-01..05

## Contexto

Requisitos não funcionais (energia, computação, resolução, profundidade de cor, entradas) e a
situação narrativa (modus, onus, tranquillitas, curvatura) determinam juntos a forma visível. Uma
tabela fixa de regimina por faixa de energia não capta a combinação e cresce sem limite com as
plataformas.

## Decisão

Uma camada única, `consilium`, escolhe o `Regimen` por busca satisfatória com recozimento
simulado sobre uma função de valor semiótico sujeita a um orçamento derivado de energia e
computação, com forçamentos narrativos explícitos. A busca é determinística e usa o gerador de
Galois do próprio estado. O hospedeiro reavalia quando mudam as facultates, a faixa de energia,
o modus, o orçamento calculado ou a chave semântica de REQ-C-05. Ele compara essas condições a
cada medida e após a evolução do mundo; a busca ocorre nas transições, sem execução
incondicional em todo tick.

## Consequências

- O stilus (giz, mosaico, letras, bicolor) é uma variável do mundo, não do hospedeiro.
- As tabelas de custo derivam de medições registradas no PR do checkpoint 4 e são a única parte
  do consilium que muda com novas medições.
- A mesma situação produz a mesma forma em hospedeiros com as mesmas facultates.
- Uma mudança de orçamento dentro da mesma faixa ou de condição narrativa pode renovar o
  regimen; a comparação das condições é distinta da execução da busca.
