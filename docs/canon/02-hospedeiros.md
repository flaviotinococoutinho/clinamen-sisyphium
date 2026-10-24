# III — Limites dos hospedeiros

Esta parte indexa obrigações dos hospedeiros. A presença de uma obrigação NÃO DEVE ser
interpretada como declaração de implementação. O [estado da entrega](../../README.md#estado)
e as [metas da visão](../01-visao.md#4-metas-da-versão-01) delimitam o que está entregue.
A [autoridade das fontes](README.md#autoridade-das-fontes) rege estes compromissos.

## CAN-H-01 — Medidas do meio e superfície

O hospedeiro DEVE medir o meio e traduzir suas capacidades e medidas em `Facultates` e
`Indicia`. Versões e APIs DEVEM chegar ao núcleo como capacidades, nunca como nomes de
plataformas. O hospedeiro DEVE fornecer os buffers de saída, chamar o núcleo e exibir
o resultado segundo a representação contratada.
Dimensões, formato e capacidade real da superfície DEVEM corresponder à saída solicitada.
Sondagem de plataforma e E/S permanecem no hospedeiro.

Fontes: [REQ-C-01](../03-especificacao.md#consilium);
[REQ-H-01 e REQ-H-02](../03-especificacao.md#limiar);
[Arquitetura §4](../04-arquitetura.md#4-contrato-de-hospedeiro) e
[§8](../04-arquitetura.md#8-hospedeiros);
[ADR 0003](../adr/0003-hospedeiros-finos-entrega-estatica.md).

## CAN-H-02 — Evolução, apresentação e transições

O hospedeiro DEVE distinguir a cadência de evolução da cadência de apresentação.
Ele DEVE comparar as condições de reavaliação de `consilium` a cada medida e após a
evolução do mundo. A comparação inclui capacidades, faixa de energia, `modus`, orçamento
calculado e chave semântica. A busca NÃO DEVE ocorrer incondicionalmente a cada tick.
Em oclusão, o protocolo DEVE aplicar um único `step` com `occluded = true` e suspender a
cadência. A saída por retorno do observador DEVE seguir a chamada final com `leaving`
e a apresentação previstas no contrato.

Fontes: [REQ-C-05](../03-especificacao.md#consilium);
[REQ-H-06](../03-especificacao.md#limiar);
[REQ-L6-01](../03-especificacao.md#l6--percepção);
[Arquitetura §4](../04-arquitetura.md#4-contrato-de-hospedeiro).

## CAN-H-03 — Retomada estática e tempo externo

O hospedeiro estático DEVE usar `advance` para recuperar a evolução decorrida.
`tabula` DEVE obter o tempo decorrido pelo `mtime` do próprio arquivo de estado, como
medida externa. Essa medida NÃO DEVE acrescentar campo nem arquivo ao estado persistido.
Com `mtime` ausente ou não confiável, a retomada DEVE usar zero ticks, salvo contagem
explícita. Época sintética DEVE exigir contagem explícita e NÃO DEVE ser subtraída de
um `mtime` real. A contagem DEVE respeitar o limite de REQ-H-06.
Nenhum registro de interação DEVE ser gravado. Fórmulas temporais, saturação e limites
operacionais permanecem definidos na especificação.

Fontes: [REQ-H-06](../03-especificacao.md#limiar);
[REQ-D-02](../03-especificacao.md#direitos-e-privacidade);
[Arquitetura §§3.3–4](../04-arquitetura.md#33-persistência);
[ADR 0002](../adr/0002-estado-em-duas-palavras.md).

## CAN-H-04 — Código de plataforma e ABI

Hospedeiros PODEM usar `std` e dependências exigidas pela plataforma.
Formatos de imagem e terminal DEVEM usar codificadores próprios, sem dependências para
esses formatos. A proibição de `unsafe` do núcleo NÃO DEVE ser interpretada como
proibição de workspace.
O ABI C DEVE seguir as funções e representações documentadas na arquitetura.
Os buffers DEVEM respeitar os formatos e limites do contrato. O índice NÃO DEVE ser
usado como especificação completa de segurança de ponteiros; essas obrigações pertencem
ao contrato documentado da implementação do limiar.

Fontes: [Convenções §8](../convencoes.md#8-código);
[REQ-H-02 e REQ-H-03](../03-especificacao.md#limiar);
[Arquitetura §6](../04-arquitetura.md#6-abi-c-sisyphium-limen).
