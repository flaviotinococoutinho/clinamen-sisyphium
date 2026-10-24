# II — Invariantes do núcleo

Esta parte indexa propriedades do contrato do crate `sisyphium`.
A [autoridade das fontes](README.md#autoridade-das-fontes) define o sentido de invariante.
As permissões dos hospedeiros permanecem na [parte III](02-hospedeiros.md).

## CAN-N-01 — Núcleo inteiro e total

O crate `sisyphium` DEVE ser `no_std`, sem `alloc`, dependências, `unsafe`, ponto flutuante
ou pânico alcançável. Cada lei DEVE ser uma função total, determinística e testada.
A aritmética DEVE declarar sua saturação ou operação modular. Acesso a fatias DEVE ser
verificado. O desenho NÃO DEVE escrever além do prefixo contratado do buffer fornecido.
Essas proibições incidem sobre o núcleo; NÃO DEVEM ser estendidas ao workspace por inferência.

Fontes: [Metacognição V4](../00-metacognicao.md#3-abstratos-de-valor-ordenados);
[REQ-L0-05 e REQ-L0-06](../03-especificacao.md#l0--substrato);
[REQ-H-02](../03-especificacao.md#limiar);
[Arquitetura §7](../04-arquitetura.md#7-determinismo-e-totalidade);
[Convenções §8](../convencoes.md#8-código);
[ADR 0001](../adr/0001-nucleo-inteiro-sem-dependencias.md).

## CAN-N-02 — Estado e persistência mínimos

`Mundus` DEVE conter `Verbum` e `Memoria`, duas palavras de 64 bits, totalizando 128 bits.
O único conteúdo persistido do mundo DEVE ser o texto de 32 dígitos hexadecimais,
independente de endianness. O leiaute, a ordem dos dígitos e a normalização na leitura
DEVEM seguir as fontes. Informação derivável do tempo ou do meio DEVE ser recalculada.
O hospedeiro NÃO DEVE ampliar esse conteúdo com histórico de interações ou campos de tempo.

Fontes: [REQ-L0-01, REQ-L0-02 e REQ-L0-03](../03-especificacao.md#l0--substrato);
[REQ-D-02](../03-especificacao.md#direitos-e-privacidade);
[Arquitetura §3](../04-arquitetura.md#3-estado);
[ADR 0002](../adr/0002-estado-em-duas-palavras.md).

## CAN-N-03 — Entradas explícitas e determinismo

O núcleo NÃO DEVE ler relógio, identificar plataforma, abrir rede ou executar E/S.
Tempo, capacidades e medidas do meio DEVEM entrar pelos argumentos do contrato.
O mesmo estado e as mesmas entradas DEVEM determinar a mesma evolução e a mesma imagem,
bit a bit, entre plataformas. A geração interna de sequências DEVE decorrer das leis
e das entradas explícitas, sem fonte externa de aleatoriedade no núcleo.

Fontes: [Arquitetura §7](../04-arquitetura.md#7-determinismo-e-totalidade);
[REQ-L0-05](../03-especificacao.md#l0--substrato);
[REQ-H-04 e REQ-H-06](../03-especificacao.md#limiar);
[ADR 0001](../adr/0001-nucleo-inteiro-sem-dependencias.md);
[ADR 0003](../adr/0003-hospedeiros-finos-entrega-estatica.md).

## CAN-N-04 — Evolução distinta do desenho

`mundus` DEVE ordenar a aplicação das leis. `arbitrium`, `sensus` e `transumptio`
alteram estado e NÃO DEVEM desenhar. `concentus` e `signum` NÃO DEVEM alterar estado.
Desenhar um quadro NÃO DEVE evoluir o mundo. `step` aplica um tick; `advance` compõe
chamadas de `step` segundo o contrato temporal. A ordem interna do tick permanece
definida na arquitetura, com rastreabilidade para os requisitos de cada etapa.

Fontes: [Arquitetura §2](../04-arquitetura.md#2-módulos-do-núcleo) e
[§7.1](../04-arquitetura.md#71-ordem-do-tick);
[REQ-H-06](../03-especificacao.md#limiar).

## CAN-N-05 — Escolha da forma e acorde comum

`consilium` DEVE escolher `Regimen` pela situação narrativa, pelas capacidades e pelo
orçamento, com os forçamentos e o retorno ao piso definidos nas fontes.
Os canais de forma DEVEM compartilhar `Concentus`, calculado por função pura.
A igualdade do acorde entre stili pressupõe capacidades e situação iguais.
O cálculo DEVE usar as capacidades reais da superfície, inclusive a grade efetiva de células.
Os custos modelados orientam a escolha. Eles NÃO DEVEM ser tratados como medição de
consumo físico; medições e metas continuam regidas pelas fontes próprias.

Fontes: [REQ-C-03 e REQ-C-04](../03-especificacao.md#consilium);
[REQ-L5-11](../03-especificacao.md#l5--signo-e-forma);
[Estética §10](../05-estetica.md#10-concentus-o-acorde);
[Consilium](../06-consilium.md);
[ADR 0004](../adr/0004-consilium-entre-narrativa-e-forma.md);
[ADR 0005](../adr/0005-stili-e-linha-organica.md).
