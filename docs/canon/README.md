# Cânone

O cânone é o índice dos compromissos estáveis de Clinamen Sisyphium. As partes identificam
o que caracteriza a obra, o que o núcleo preserva e o que limita os hospedeiros. Cada
compromisso aponta para a fonte que define seu alcance e sua verificação.

## Partes

| Parte | Compromissos | Objeto |
| --- | --- | --- |
| [I — Identidade e valor](00-identidade.md) | CAN-I-01 a CAN-I-06 | Intenção, leis, interesse do observador, legibilidade, forma e expressão |
| [II — Invariantes do núcleo](01-nucleo.md) | CAN-N-01 a CAN-N-05 | Totalidade, estado, determinismo, evolução e escolha da forma |
| [III — Limites dos hospedeiros](02-hospedeiros.md) | CAN-H-01 a CAN-H-04 | Medidas, cadências, retomada, representação e buffers |

Os identificadores distinguem compromissos e permanecem estáveis. Um identificador removido
NÃO DEVE ser reutilizado. Os requisitos conservam seus identificadores `REQ-` nas fontes.
O cânone NÃO DEVE ser usado como lista exaustiva de requisitos ou prova de implementação.

## Autoridade das fontes

Somente o [harness de `01-visao.md` §1](../01-visao.md#1-harness-irrefutável) é não negociável.
Os demais compromissos permanecem revisáveis por ADR, conforme essa mesma seção.
O termo invariante designa uma propriedade preservada pelo contrato vigente.
O cânone NÃO DEVE sobrepor suas sínteses às fontes ou estabelecer outra autorização de mudança.

A consulta segue as funções abaixo:

| Fonte | Função |
| --- | --- |
| [Visão §1](../01-visao.md#1-harness-irrefutável) | Restrições não negociáveis |
| [Metacognição §§3–5](../00-metacognicao.md) | Valores, essenciais e conversão das restrições do meio em leis |
| [Especificação](../03-especificacao.md) | Comportamento verificável e rastreabilidade dos requisitos |
| [ADRs aceitos](../adr/) | Decisões e suas razões |
| [Ontologia](../02-ontologia.md) | Níveis, interfaces e grafia dos termos definidos |
| [Arquitetura](../04-arquitetura.md) | Composição, estado e interfaces |
| [Estética](../05-estetica.md) | Forma e parâmetros de composição |
| [Consilium](../06-consilium.md) | Escolha da forma sob capacidades, situação e orçamento |
| [Narrativa](../07-narrativa.md) | Corpus e regras de escrita da experiência |
| [Verificação](../08-verificacao.md) | Métodos de verificação e medições |
| [Convenções](../convencoes.md) | Escrita, código e entrega |

Uma divergência exige harmonizar as fontes afetadas, preservar o harness e registrar por ADR
as mudanças de decisão, conforme a visão. A ordem de consulta NÃO DEVE ocultar contradições.
Planos e relatórios registram trabalho e evidência. Testes demonstram cumprimento nas condições
examinadas. Nenhum desses registros substitui sozinho o contrato vigente.
Entrega e checkpoints continuam regidos por [Convenções §§4–7](../convencoes.md#4-commits).

## Parâmetros nas fontes

Os números 128 bits, duas palavras de 64 bits e 32 dígitos hexadecimais integram o compromisso
de estado e persistência. Os demais valores permanecem nas fontes que os especificam.

- Contraste, tons, paleta, dimensões, margens, glifos e limiares de conversão pertencem à
  [especificação de forma](../03-especificacao.md#l5--signo-e-forma) e à [estética](../05-estetica.md).
- Constantes de ponto fixo, polinômios, sementes e limiares narrativos pertencem à
  [especificação](../03-especificacao.md) e às respectivas leis.
- Bandas de energia, cadências, propostas, pesos, custos e orçamento pertencem à
  [especificação de consilium](../03-especificacao.md#consilium) e a [Consilium](../06-consilium.md).
- Resoluções de referência, quadros-ouro, hashes, metas de consumo, tempos e tamanhos de
  binário pertencem à [verificação](../08-verificacao.md) e à [especificação](../03-especificacao.md).
- Versões de compilador e APIs, leiautes de interface e limites operacionais de `advance`
  pertencem às [convenções](../convencoes.md), à [arquitetura](../04-arquitetura.md) e à
  [especificação do limiar](../03-especificacao.md#limiar).

A ausência desses parâmetros no índice NÃO DEVE ser interpretada como dispensa. Cada valor
continua obrigatório em sua fonte enquanto vigente.
