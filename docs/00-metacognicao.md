# 00 — Metacognição: pré-planejamento

## 1. Finalidade

Este documento precede o plano. Fixa os abstratos de valor que vinculam as decisões seguintes,
separa o essencial do acidental e define o critério de sinergia entre ambos. Toda decisão
posterior DEVE citar o abstrato de valor que a justifica. Uma decisão sem abstrato de valor é
removida.

## 2. Método aplicado

1. Axiologia: enumerar o que tem valor, para quem e em que horizonte; ordenar; definir um teste de
   aderência por abstrato.
2. Essencial e acidental (Brooks, 1986): classificar cada elemento candidato; marcar o que o
   conceito propõe converter de acidental em essencial.
3. Lei de conversão: aceitar uma restrição acidental no núcleo somente quando existe um
   isomorfismo explícito entre a restrição e uma lei do mundo simulado; sem isomorfismo, a
   restrição fica no hospedeiro.
4. Estratificação ontológica: atribuir cada mecanismo a um nível e declarar as interfaces entre
   níveis (`02-ontologia.md`).
5. Pré-mortem (Klein, 2007): listar os modos de falha com a obra funcionando; derivar antimetas.
6. Densidade de valor: para cada componente, registrar o significado que carrega e o custo que
   impõe; eliminar o que tem custo sem significado.
7. Inversão: listar o que tornaria a obra derivativa, vazia ou danosa; derivar restrições.
8. Travessia de culturas: para cada nível, buscar ao menos uma fonte fora da tradição
   greco-latina moderna; aceitar a fonte somente quando ela descreve o mesmo mecanismo, não quando
   apenas o decora.

## 3. Abstratos de valor (ordenados)

| # | Abstrato | Definição operacional | Teste de aderência |
| --- | --- | --- | --- |
| V1 | Exigência de pensamento | A imagem contém relações que só se resolvem por inferência do observador. Nenhuma legenda explica o estado. | Para cada estado visível existe ao menos uma leitura que exige conhecer uma lei do mundo. Nenhum texto de interface descreve o mecanismo. |
| V2 | Canonicidade | Cada mecanismo tem nome e fonte na teoria que o originou: filosofia, matemática, física, ciência da computação, ciência cognitiva, estética; de mais de uma cultura e época. | O termo consta no glossário com a referência. O nome no código é o termo latino ou a transliteração canônica. |
| V3 | Originalidade | Nenhum ativo, nome, texto ou mecânica copiado de produto existente. A obra histórica de referência orienta a ambição, não a forma. | Varredura do repositório: nenhum nome de produto, marca ou motor de terceiros no núcleo, nos ativos e nos textos. |
| V4 | Formalidade | Cada lei é uma função total, determinística e testada. O estado tem leiaute de bits documentado e invariantes verificados. | Requisito, módulo e teste rastreáveis em `03-especificacao.md`. Integração contínua sem falhas. |
| V5 | Economia | Estado em 128 bits. Núcleo sem alocação, sem ponto flutuante, sem dependências. Custo por quadro decresce com a escassez de energia. | Medições de tamanho de estado, de binário e de tempo por quadro por regimen registradas em cada checkpoint. |
| V6 | Presença | A obra existe como fundo de tela onde houver tela: imagem estática regenerada, animação em terminal, navegador, papel de parede vivo. | Um núcleo; hospedeiros finos; cada hospedeiro declara facultates e recebe um regimen. |
| V7 | Discrição | A obra fala por seus processos, línguas e referências. Nenhuma assinatura, pessoa, ferramenta ou marca aparece no núcleo, nos textos e nos artefatos. | Varredura de textos e metadados. Autoria apenas no registro de commits da conta. |
| V8 | Sinergia | Nenhum nível isolado produz a obra. A medida da obra é a soma qualitativa da sensibilidade pensante que os níveis combinados provocam. | Cada mecanismo tem entrada em pelo menos dois níveis (origem e manifestação). Nenhum mecanismo existe em um único nível. |

Não são valores: engajamento, retenção, viralidade, contagem de funcionalidades, fotorrealismo,
efeitos de GPU, humor, conforto explicativo, recompensa hedônica.

## 4. Essencial e acidental

### 4.1 Essencial

Complexidade do problema. Permanece em qualquer implementação.

- E1 A ascensão repetida e a consciência da repetição (Camus, 1942; Eclesiastes 1:9).
- E2 O clinamen: desvio mínimo em tempo e lugar incertos que ramifica consequências
  desproporcionais (Lucrécio, II, 216–293; Lorenz, 1963).
- E3 A finitude do mundo: o cosmos é um número; o que acontece é o desdobramento determinístico
  desse número (Laplace, 1814; Kolmogorov, 1965; Leibniz, 1703).
- E4 O observador implicado: observar interrompe; interromper tem consequência; o observador é
  parte do sistema (von Foerster, 1974; Brecht, 1936).
- E5 O sonho e o reconhecimento: parte do que se vê não aconteceu; a revelação é retroativa
  (Zhuangzi, cap. 2; Aristóteles, _Poética_ 11).
- E6 A tríade do signo: cada estado se mostra por semelhança, por conexão física e por convenção
  (Peirce, CP 2.247–249).
- E7 O peso: a escolha acumula massa; a massa altera a física e a forma (Espinosa, _Ética_ III;
  _Livro dos Mortos_, cap. 125; _Bhagavad Gita_ 2.47).
- E8 A originação dependente: cada consequência nasce de condições, nenhuma de um ato isolado
  (_Samyutta Nikaya_ 12.21; Hume, 1748).

### 4.2 Acidental

Complexidade da implementação. Varia com o meio.

- A1 Largura da palavra de máquina; formato de pixel; profundidade de cor; resolução; taxa de
  atualização.
- A2 Energia disponível: bateria, estado térmico, política de economia.
- A3 Relógio e localização: hora, data, hemisfério, fuso.
- A4 Entradas: toque, ponteiro, acelerômetro, troca de página, oclusão por aplicativos.
- A5 Versão do sistema operacional e APIs: serviço de papel de parede, canvas acelerado, terminal,
  navegador.
- A6 Persistência: arquivo, preferências, nenhuma.
- A7 Capacidade de cálculo: CPU, ausência de FPU, orçamento por quadro.
- A8 Som: presente ou ausente.

## 5. Lei de conversão e tabela de isomorfismos

Regra: uma restrição acidental entra no núcleo somente como lei do mundo com nome, domínio,
contradomínio e teste. Sem isomorfismo, a restrição fica no hospedeiro (`hospes`) e o núcleo não a
conhece.

| Acidental | Lei do mundo | Nível | Mecanismo |
| --- | --- | --- | --- |
| A1 largura da palavra (64 bits) | Finitude do cosmos: `Verbum` (64 bits) e `Memoria` (64 bits) | L0 | leiaute de bits; gerador de Galois; rocha como leitura direta do estado |
| A2 energia (0..=255) | Temperatura termodinâmica; cadência por regimen (Simon, 1955; Kirkpatrick et al., 1983) | L1 | `consilium` escolhe o regimen; a cadência cai com a energia |
| A3 relógio | Efeméride: fase circadiana, estação, fase lunar, solstícios e equinócios | L1 | aritmética modular sobre segundos de época |
| A4 toque e movimento | Sismo; vento; oráculo do polegar | L1, L7 | injeção de ruído; cadeia de Markov de 32 bits |
| A4 oclusão | Sacada artificial: mutação cega à mudança (Rensink et al., 1997) | L6 | mutação somente em oclusão |
| A5 APIs e versões | Facultates do hospedeiro; espaço de regimina | consilium | vetor de facultates → regimen e stilus |
| A1 resolução | Spatium: composição por φ; estratos de paralaxe | L2 | proporções por constantes irracionais |
| A6 persistência | Memoria: uma palavra de 64 bits; ruínas por contagem de bits | L8 | texto de 32 dígitos hexadecimais |
| A7 ausência de FPU | Aritmética de ponto fixo como física do mundo | L0, L2 | Q16.16; CORDIC; seno de Bhaskara I; raiz inteira |
| A8 som ausente | Mundo mudo (0.1); som como índice no futuro | L5 | adiado |

## 6. O meio entre narrativa e forma

Entre o estado narrativo (L3–L8) e a forma visível (L5) existe uma camada de escolha:
`consilium`. Ela recebe facultates do hospedeiro, requisitos não funcionais (energia, orçamento de
cálculo, resolução, profundidade de cor) e a situação narrativa (modus, onus, tranquillitas,
curvatura), e escolhe o regimen: stilus (giz vetorial, mosaico, letras, bicolor), número de
estratos, cadência, camadas ativas. A escolha é uma busca satisfatória (Simon, 1955) com
recozimento simulado (Kirkpatrick et al., 1983) sobre uma função de valor semiótico sujeita a
custo. A situação narrativa PODE forçar um stilus. Detalhes em `06-consilium.md`.

## 7. Pré-mortem e antimetas

| Falha com a obra funcionando | Antimeta |
| --- | --- |
| F1 Pretensão sem substância: termos canônicos fora do sentido original | Nenhum termo sem fonte e sem lei implementada |
| F2 Opacidade total: nada reconhecível; abandono | Em todo regimen, a montanha, a figura e a rocha são identificáveis em menos de 1 s de observação |
| F3 Ruído decorativo: efeitos sem significado | Cada efeito tem requisito com o significado que carrega |
| F4 Custo de energia acima do fundo de tela padrão | Regimen mínimo com quadro estático e custo de CPU abaixo de 0,1 % |
| F5 Derivação: pastiche, nomes de motores, ativos de terceiros | Varredura de originalidade por checkpoint |
| F6 Vocabulário vazio: adjetivos sem medida | Linguagem controlada (`convencoes.md`) |
| F7 Escopo infinito | Checkpoints com entregas verificáveis em PRs; versão 0.1 fechada em `01-visao.md` |
| F8 Significado preso a uma API | Núcleo `no_std`; hospedeiros finos |
| F9 Captura de atenção: a obra aumenta o uso do aparelho | Nenhum mecanismo recompensa a interação; a calma estabiliza o mundo; nenhuma notificação |
| F10 Dano ao observador: manipulação, dados, consumo | Harness irrefutável (`01-visao.md` §1) |

## 8. Decisões derivadas

| Decisão | Valor |
| --- | --- |
| Nome próprio `Clinamen Sisyphium` (forma curta `sisyphium`); termo `clinamen` para o evento de desvio mínimo | V2, V3 |
| Vocabulário externo em latim; documentação em português; instruções de código em inglês | V2, V7 |
| Núcleo em Rust `no_std`, sem alocação, sem ponto flutuante, sem dependências, sem `unsafe` | V4, V5, V6 |
| Estado = `Verbum` (64 bits) + `Memoria` (64 bits); persistência = 32 dígitos hexadecimais | V5, V1 |
| Hospedeiros finos: imagem estática (`tabula`), terminal (`textus`), ABI C (`limen`), navegador (`tela`) | V6 |
| Primeira entrega: fundo de tela estático regenerado por agendamento no macOS, Linux e Windows com o mesmo binário | V6 |
| `consilium` como camada entre narrativa e forma | V5, V1, V8 |
| Stili: giz vetorial, mosaico, letras, bicolor; linha orgânica como princípio (Art Nouveau e suas raízes japonesas) | V1, V3, V2 |
| Canonicidade entre culturas e épocas: cada nível com fonte ocidental e oriental quando o mecanismo coincide | V2, V8 |
| Sem som na 0.1 | V5 |
| Repositório público para estudo; reprodução reservada; licença própria sob a lei brasileira | V7, harness |

## 9. Suposições declaradas

Decisões tomadas sem consulta. Cada uma é reversível por ADR.

- S1 Hemisfério padrão: sul; ajustável pela facultas `australis`.
- S2 O núcleo rasteriza na resolução do hospedeiro; o fator inteiro s escala figuras e linhas; o
  stilus Musivum rasteriza em W/s × H/s e amplia por vizinho mais próximo. Largura lógica mínima:
  272 px.
- S3 Fonte própria: maiúsculas com diacríticos do português, dígitos, pontuação, símbolos de
  lógica modal e letras gregas de constantes; célula de 6×10 px (glifo 5×7, 2 linhas de
  diacrítico, 1 de espaço).
- S4 O corpus (crônicas, diálogos, aforismos) é escrito dentro do projeto, no checkpoint que
  implementa `arbitrium`, sob as regras de `07-narrativa.md` §3; nenhum texto externo é
  reutilizado.
- S5 Calendário festivo = calendário astronômico (solstícios, equinócios, luas nova e cheia).
  Nenhum feriado civil ou comercial.
- S6 Língua padrão da experiência: inglês; português selecionável; os dois corpora existem.
- S7 Um tick = uma chamada de `step`. Hospedeiros vivos chamam `step` a cada `tick_ms` do
  regimen. Hospedeiros estáticos recuperam o tempo decorrido desde a última execução com
  `advance` (REQ-H-06). A escassez de energia dilata o tempo subjetivo do mundo por `tick_ms`.
