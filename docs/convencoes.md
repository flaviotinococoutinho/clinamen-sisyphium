# Convenções

## 1. Línguas

| Camada | Língua | Exemplos |
| --- | --- | --- |
| Nome do projeto, crates, módulos, tipos, campos de estado, termos definidos | Latim | `clinamen-sisyphium`, `sisyphium`, `verbum`, `memoria`, `Regimen`, `conatus`, `onus` |
| Documentação, ADRs, planos, corpos de PR, textos de release | Português (pt-BR) | este documento |
| Instruções de código: funções, variáveis locais, documentação de API, mensagens de commit | Inglês | `fn step`, `let interval_ms`, `/// Advances the world by one tick.` |
| Texto da experiência (crônicas, diálogos, aforismos) | Inglês e português, selecionável pelo hospedeiro | `annales_en.rs`, `annales_pt.rs` |

## 2. Estilo da prosa

- Frio e direto. Uma frase, uma ideia. Voz ativa com sujeito explícito. Presente do indicativo.
- Sem pronomes de primeira e segunda pessoa. Sem figuras de linguagem, aforismos, citações
  culturais ou fórmulas de cortesia. Duas exceções: a seção "Efeito sobre a experiência" de cada PR
  e o texto da obra registrado em `07-narrativa.md`.
- Sem menção a pessoas, ferramentas, agentes ou processos de produção do texto e do código. A
  autoria registrada é a conta do repositório.
- Verbos normativos em maiúsculas com o sentido da RFC 2119: DEVE, NÃO DEVE, PODE, RECOMENDA-SE.
- Termos definidos têm grafia única e constam do glossário em `02-ontologia.md`. Sinônimos para
  termos definidos são proibidos.
- Faixas fechadas na notação `a..=b`. Unidades explícitas. Vírgula decimal na prosa, ponto no
  código.
- Teorias e fontes são citadas pelo nome do autor e ano, com entrada em `referencias.md`.
- Sem nomes de doutrinas, métodos ou marcas de engenharia de software. Uma propriedade é descrita
  pela propriedade ("função total e determinística"), não por um rótulo de mercado.
- Os documentos descrevem o estado presente. Histórico de mudanças, desvios, contornos, falhas e
  evidências de verificação ficam exclusivamente em commits, PRs e releases. Um documento que muda
  é reescrito; o histórico fica no controle de versão.

## 3. Densidade canônica do código

- Cada função implementa uma lei nomeada e documentada com a fonte. Não existe código sem lei.
- Estado empacotado em palavras de máquina; campos extraídos por deslocamento e máscara; tabelas
  pequenas empacotadas em inteiros; seleção sem desvio condicional quando a seleção é a própria lei.
- Preferência por forma fechada sobre armazenamento: posições por dispersão multiplicativa,
  sequências por geradores, texturas por leitura direta do estado.
- Aritmética inteira e de ponto fixo. Sem ponto flutuante no núcleo.
- Sem camadas, interfaces ou abstrações sem teorema que as exija. Sem padrões de projeto nomeados.
- Toda constante tem derivação registrada (fórmula, precisão, fonte) e teste que a confere.
- Toda lei tem ao menos um teste nomeado pela lei. Testes de propriedade usam o próprio gerador
  caótico do núcleo como fonte de casos.

## 4. Commits

- Formato `tipo(escopo): assunto`, em inglês, minúsculas, sem ponto final, imperativo.
- Tipos: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `build`, `ci`, `chore`.
- Escopos: `verbum`, `memoria`, `natura`, `ephemeris`, `consilium`, `spatium`, `corpus`,
  `arbitrium`, `concentus`, `signum`, `sensus`, `transumptio`, `hospes`, `mundus`, `tabula`, `textus`, `limen`,
  `tela`, `hosts`, `docs`, `spec`, `release`.
- Corpo opcional em português, estilo frio: o que mudou e a evidência.
- Sem trailers de coautoria, de ferramentas ou de sessões.
- Um commit por mudança coerente. Os commits de um PR contam a cadeia de implementação em ordem.

## 5. Pull requests

- Um PR por checkpoint ou por mudança isolada. Base `main`. Ramo `tipo/assunto-curto`.
- Sem PRs empilhados: cada PR nasce de `main` após o merge do anterior.
- Merge por rebase. Histórico linear. Ramo apagado após o merge.
- Título igual ao assunto do commit principal.
- Corpo obrigatório com as seções abaixo, nesta ordem e sem outras.

```
## Substrato
Fatos medidos antes da mudança: versões, tamanhos, tempos, contagens, estado dos testes.

## Cadeia empírica
Lista numerada. Cada item: observação → hipótese → decisão → implementação → medição,
com o dado que sustenta cada passo.

## Alterações
Módulos, arquivos, invariantes, requisitos (REQ-) cobertos, testes acrescentados. Contagens.

## Verificação
Comandos executados e resultados literais, resumidos com números.

## Riscos residuais
O que não foi verificado e por quê. O que pode falhar e como se detecta.

## Efeito sobre a experiência
O que o observador passa a perceber. Única seção com linguagem quente.
```

## 6. Checkpoints

- Checkpoint = PR mesclado em `main` + tag anotada `cp/N-assunto`. A evidência de verificação
  (comandos e resultados) fica no corpo do PR e na mensagem da tag.
- A partir do checkpoint 1, um checkpoint exige `cargo fmt --check`,
  `cargo clippy --workspace --all-targets -- -D warnings` e `cargo test --workspace` sem falhas.
- Cada checkpoint registra no PR as medições: tamanho do estado, tamanho dos binários, tempo por
  quadro por regimen e por stilus na máquina de referência.
- O `README.md` indica apenas o checkpoint presente.
- Planos em `docs/planos/` descrevem trabalho futuro; o PR que conclui um plano o remove, e o
  corpo do PR carrega a evidência.

## 7. Versionamento

- Versionamento semântico 2.0.0 para o workspace. Tags `vX.Y.Z` em commits de `main`.
- As notas de versão ficam no texto da release, com as mesmas seções do corpo de PR. Nenhum
  arquivo de histórico no repositório.
- Antes de 1.0.0, mudanças incompatíveis no contrato de hospedeiro ou no leiaute de estado
  incrementam a versão menor e são registradas em ADR.

## 8. Código

- Rust, edição 2024, versão mínima 1.85.
- Crate `sisyphium` (núcleo): `#![no_std]`, sem `alloc`, sem dependências, sem `unsafe`, sem ponto
  flutuante (`clippy::float_arithmetic` proibido), sem pânico alcançável (aritmética saturante ou
  modular explícita; acesso a fatias verificado), `#![deny(missing_docs)]`.
- Hospedeiros: `std` permitido; dependências permitidas apenas quando a plataforma as exige;
  nenhuma dependência para formatos de imagem ou terminal (codificadores próprios).
