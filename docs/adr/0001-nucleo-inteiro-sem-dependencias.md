# ADR 0001 — Núcleo inteiro, sem alocação, sem dependências, sem `unsafe`

- **Status:** aceito
- **Relacionados:** `01-visao.md` §7–8; `03-especificacao.md` REQ-L0-06; `convencoes.md` §3 e §8

## Contexto

O núcleo precisa ser determinístico bit a bit entre plataformas, executável em dispositivos sem
unidade de ponto flutuante, verificável por lints e testes, e portável para alvos nativos,
Android e wasm sem adaptação. Ponto flutuante introduz diferenças entre arquiteturas e modos de
arredondamento; alocação dinâmica introduz falhas de memória e tempo não determinístico;
dependências introduzem código fora do controle da obra e nomes de terceiros.

## Decisão

O crate `sisyphium` é `#![no_std]`, sem `alloc`, sem dependências, com `unsafe_code` proibido,
`clippy::float_arithmetic` proibido, aritmética explícita (saturante ou modular) e acesso a
fatias verificado. Toda a geometria usa ponto fixo Q16.16 em `i32` com intermediários `i64`; a projeção de
curvatura opera sobre coordenadas normalizadas em Q8.8, convertidas por deslocamento exato.
Funções trigonométricas usam a aproximação de Bhaskara I e CORDIC. O estado inteiro cabe em
128 bits e nunca é alocado.

## Consequências

- Os mesmos quadros-ouro valem em Linux, macOS, Android e wasm.
- O custo por quadro é previsível e medido por stilus.
- A precisão do seno é 0,0017 absoluto; a composição tolera esse erro por construção.
- Os hospedeiros fazem toda a E/S, inclusive a codificação de imagem, com codificadores próprios.
