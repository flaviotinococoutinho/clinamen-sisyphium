# ADR 0002 — Estado do mundo em duas palavras de 64 bits

- **Status:** aceito
- **Relacionados:** `02-ontologia.md` §3; `04-arquitetura.md` §3; REQ-L0-01..04

## Contexto

A obra afirma a finitude do mundo como lei: o cosmos é um número. A persistência precisa ser
mínima, legível, independente de plataforma e sem dados pessoais. A textura da rocha é a leitura
direta do estado, o que exige que o estado seja uma palavra de máquina.

## Decisão

O estado é `Verbum` (presente) e `Memoria` (passado e intenção), duas palavras de 64 bits com
leiaute fixo documentado. Todo padrão de bits é um mundo válido, com a única exceção de
`semen` = 0, normalizado para `0xACE1`. O único conteúdo persistido do mundo é o texto de
exatamente 32 dígitos hexadecimais. A data de modificação do próprio arquivo de estado é uma
medida externa do sistema de arquivos, lida pelo hospedeiro estático para recuperar o tempo
decorrido (REQ-H-06); ela não acrescenta campo ao estado nem exige outro arquivo.

## Consequências

- O número de mundos possíveis é 2¹²⁸ − 2¹¹². A alma do mundo é transcrita em 32 dígitos.
- Toda informação derivável do tempo ou do meio (fase circadiana, regimen, postura) é recalculada
  e nunca armazenada.
- O hospedeiro estático usa zero ticks quando a data de modificação está ausente ou não é
  confiável, salvo contagem explícita. Época sintética exige contagem explícita e não se combina
  com a data de modificação real (REQ-H-06).
- Mecanismos que exigiriam memória adicional (histórico de intervalos de interação) são medidos
  pelo hospedeiro e entregues como `Indicia`, ou reformulados sem estado (ruído 1/f como função do
  tempo; eventos raros como função de janela temporal).
