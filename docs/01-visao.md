# 01 — Visão

## 1. Harness irrefutável

Único conjunto de restrições não negociáveis. Todo o resto é revisável por ADR.

| Domínio | Regra |
| --- | --- |
| Moral | A obra NÃO DEVE agir contra o interesse do observador. Nenhum mecanismo de percepção (surpresa, tensão de Zeigarnik, eventos raros) DEVE aumentar o uso do aparelho. A obra recompensa a calma: a ausência de interação estabiliza o mundo. |
| Ético | Nenhum padrão de captura: sem notificações, sem pedidos de interação, sem contadores, sem progresso exibido, sem escassez artificial com finalidade de retenção. A obra não se beneficia da atenção. |
| Legal | Direitos autorais e de programa de computador (Lei nº 9.610/1998; Lei nº 9.609/1998). Dados pessoais (Lei nº 13.709/2018): nenhum dado sai do aparelho; nenhum dado é armazenado além de 128 bits de estado; a interação é reduzida a medidas de entropia no hospedeiro e descartada. Nenhum ativo de terceiros. |
| Produtos e marcas | Nenhum nome de produto, marca, motor, loja, rede ou serviço no núcleo, nos textos da obra, nos ativos e nos artefatos. Na documentação e nos scripts de hospedeiros, nomes de sistemas operacionais e de utilitários do sistema aparecem apenas como instrução técnica indispensável. Nenhuma recompensa hedônica, nenhuma monetização, nenhum elemento de jogo com pontuação. |

## 2. Intenção

Construir um fundo de tela dinâmico cujo conteúdo é uma simulação finita, determinística e
verificável do mito de Sísifo, na qual as restrições do meio de execução operam como leis do
mundo simulado, e cuja experiência exige pensamento do observador.

Princípio de sinergia: nenhum nível isolado produz a obra. A medida da obra é a soma qualitativa
máxima da sensibilidade pensante que os níveis combinados provocam: percepção, inferência,
reconhecimento e dúvida.

## 3. Por quê

- O protetor de tela perdeu a função de proteção do hardware. Painéis LCD não sofrem queima de
  fósforo; painéis OLED usam deslocamento de pixel e corte de sinal. O fundo de tela é a superfície
  de computação calma mais presente (Weiser e Brown, 1996).
- A obra histórica de referência do gênero (1992; `referencias.md`) demonstrou densidade de
  acontecimentos em hardware limitado. Seus direitos permanecem reservados; nada dela é reutilizado.
- Existe espaço para uma obra cujo valor é a filosofia aplicada em código: teorias nomeadas em
  todos os níveis ontológicos, de mais de uma cultura, sem vocabulário de produtos.

## 4. Metas da versão 0.1

- M1 Núcleo `sisyphium` com os módulos `verbum`, `memoria`, `natura`, `ephemeris`, `consilium`,
  `spatium`, `corpus`, `arbitrium`, `concentus`, `signum`, `sensus`, `transumptio`, `hospes`, `mundus`
  (`04-arquitetura.md`).
- M2 Hospedeiro `sisyphium-tabula`: linha de comando que gera um quadro (PPM e PNG) para tempo,
  energia, sismo, língua e facultates dados; scripts de papel de parede para macOS (launchd e
  osascript), Linux (feh ou swww) e Windows (PowerShell), com o mesmo binário.
- M3 Hospedeiro `sisyphium-textus`: animação em terminal; stilus de letras nativo e raster por
  meio-bloco.
- M4 Hospedeiro `sisyphium-limen`: ABI C (`extern "C"`) para papéis de parede vivos e para
  exportação wasm; verificado por testes no próprio crate.
- M5 Hospedeiro `sisyphium-tela`: página estática que carrega o wasm e desenha em canvas;
  condicional à disponibilidade do alvo `wasm32-unknown-unknown`.
- M6 Documentação completa; integração contínua; checkpoints com evidência; versão 0.1.0
  etiquetada.

## 5. Não-metas da versão 0.1

Som. Hospedeiro Android compilado. Hospedeiro iOS. Configuração por interface gráfica. Rede.
Telemetria para fora do aparelho. Línguas da experiência além de inglês e português.

## 6. Antimetas

- Texto de interface que explique o mecanismo ao observador.
- Dependência de GPU, shader ou biblioteca gráfica no núcleo.
- Nome, ativo, fonte ou texto de terceiros.
- Contador visível: pontuação, estatística, progresso.
- Consumo de energia acima do fundo de tela estático do sistema no regimen mínimo.
- Nome de doutrina de engenharia no código ou na documentação.

## 7. Restrições

- Legais: originalidade total; licença de estudo com reprodução reservada (`LICENSE`).
- Técnicas: núcleo `no_std`, sem `alloc`, sem dependências, sem `unsafe`, sem ponto flutuante, sem
  pânico; determinismo bit a bit entre plataformas; estado persistido em texto hexadecimal
  independente de endianness.
- Estéticas: `05-estetica.md`.
- Autoria e línguas: `convencoes.md` §1–3.

## 8. Decisões

| # | Decisão | Razão |
| --- | --- | --- |
| D1 | Núcleo em Rust, edição 2024 | `no_std`; ausência de comportamento indefinido em código seguro; alvos nativos, Android e wasm; lints de restrição (`float_arithmetic`) verificam a lei de aritmética inteira |
| D2 | Estado em dois registradores de 64 bits (`04-arquitetura.md` §3) | V5; a rocha é a leitura direta do `Verbum`; a alma do mundo cabe em 32 dígitos |
| D3 | Primeira entrega como imagem estática regenerada por agendamento | Modo de fundo de tela disponível em todo sistema operacional sem API própria; o regimen de menor energia já é uma imagem estática |
| D4 | Stili selecionados por `consilium`: `Creta` (giz vetorial em 16 tons), `Musivum` (mosaico: raster em baixa resolução com figuras), `Littera` (células de glifos), `Bicolor` (1 bit por matriz de Bayer) | A situação narrativa e os requisitos não funcionais escolhem a forma; a linha orgânica é o princípio comum |
| D5 | Dois corpora da experiência: inglês (padrão) e português | Convenção de línguas; presença global; língua do autor |
| D6 | Fonte própria embutida | Nenhum ativo de terceiros; diacríticos do português; símbolos de lógica modal |
| D7 | Sem som na 0.1; eventos sonoros definidos como índices no estado | Escopo; o índice sonoro nasce da mesma lei que o visual |
| D8 | Checkpoints por PR; merge por rebase; sem PRs empilhados | Histórico linear com evidência por etapa |
| D9 | Canonicidade entre culturas: cada nível cita fonte ocidental e oriental quando o mecanismo coincide; a presença oriental é estrutural e discreta (hexagrama de 6 bits, ensō, intervalo, kintsugi, sonho de Zhuangzi, originação dependente) | V2, V8; nenhum exotismo decorativo |

## 9. Entregas por versão

| Versão | Conteúdo |
| --- | --- |
| 0.1.0 | M1–M6 |
| 0.2.0 | Hospedeiro Android (Kotlin sobre `limen`); som como índice (síntese granular inteira); medições em aparelhos |
| 0.3.0 | Painéis dedicados (tinta eletrônica, barra); ajustes por medições em campo |

## 10. Checkpoints da versão 0.1

| Checkpoint | Conteúdo | Tag |
| --- | --- | --- |
| 0 | Fundação documental | `cp/0-fundatio` |
| 1 | Substrato e natureza: `verbum`, `memoria`, `natura`, `ephemeris`, `spatium`, `hospes`; `mundus` com genesis, format, parse; integração contínua | `cp/1-substratum` |
| 2 | Signo e corpo: `concentus` (acorde e paleta), `signum` (pixel, fonte, primitivas, stili), `corpus` (figuras, rocha), composição; quadros-ouro iniciais | `cp/2-signum` |
| 3 | Escolha, percepção e narração: `arbitrium` com corpus em duas línguas, `sensus`, `transumptio`, `consilium`; `mundus::step`, `render`, `advance` | `cp/3-arbitrium` |
| 4 | Hospedeiros: `sisyphium-tabula`, `sisyphium-textus`, `sisyphium-limen`, scripts de papel de parede; medições que fixam as tabelas de custo de `consilium` | `cp/4-hospites` |
| 5 | `sisyphium-tela` (wasm), comparação de hashes entre sistemas no CI, versão 0.1.0 | `cp/5-tela` |

## 11. Delegado ao construtor

- Valores das tabelas de custo e de valor semiótico de `consilium`, documentados em tabela.
- Desenho das figuras dentro das regras de `05-estetica.md`.
- Ordem interna de implementação dos módulos.
- Formato interno da tabela de glifos.
