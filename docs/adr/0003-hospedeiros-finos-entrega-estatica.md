# ADR 0003 — Hospedeiros finos e entrega estática como primeira presença

- **Status:** aceito
- **Relacionados:** `01-visao.md` §4 e §8; `04-arquitetura.md` §4 e §8

## Contexto

A obra precisa existir como fundo de tela em sistemas sem API de papel de parede vivo e em
sistemas com ela. O modo de menor energia da obra já é uma imagem estática. Cada plataforma
expõe medidas do meio (energia, versão, dimensões, entradas) por mecanismos próprios.

## Decisão

O núcleo não conhece plataformas. Cada hospedeiro mede o meio, traduz versões e APIs em
`Facultates`, chama `consilium`, `step` e `render`, e exibe. A primeira presença é o hospedeiro
`sisyphium-tabula`, que gera uma imagem e é agendado pelo sistema (launchd, systemd, agendador de
tarefas) para regenerar o fundo de tela. Hospedeiros vivos (terminal, navegador, ABI C) usam o
mesmo núcleo e o mesmo contrato.

## Consequências

- Um binário serve macOS, Linux e Windows como fundo de tela estático regenerado.
- Papéis de parede vivos (Android e outros) ligam-se ao ABI C sem alterar o núcleo.
- A sondagem de sistema fica confinada aos hospedeiros; o núcleo nunca recebe nomes de sistemas.
