# 02 — Ontologia

## 1. Níveis

Cada mecanismo da obra pertence a um nível de origem e a ao menos um nível de manifestação. Cada
nível tem teorias canônicas de mais de uma cultura quando o mecanismo coincide. A tabela nomeia o
módulo que implementa o nível.

| Nível | Nome | Domínio | Teorias canônicas | Módulo |
| --- | --- | --- | --- | --- |
| L0 | Substrato | O cosmos como número: duas palavras de 64 bits, seus campos e suas leis de empacotamento | Shannon, 1948; Kolmogorov, 1965; Laplace, 1814; Leibniz, 1703 (aritmética binária e os hexagramas de Fuxi); Golomb, 1967 (registradores de deslocamento) | `verbum`, `memoria` |
| L1 | Natureza e tempo | Forças, caos, efeméride, temperatura | Lucrécio, II (semina rerum, clinamen); Wolfram, 1983 (regra 30); May, 1976 e Feigenbaum, 1978 (mapa logístico); Hénon, 1976; Voss e Clarke, 1978 (ruído 1/f); Metropolis et al., 1953; Kirkpatrick et al., 1983; Simon, 1955; Meeus, 1991 (efemérides); _Dhammapada_ 277 (impermanência); Kamo no Chōmei, 1212 (_Hōjōki_, o rio) | `natura`, `ephemeris`, `consilium` |
| L2 | Espaço | Ponto fixo, projeções, curvatura, estratos, figuras impossíveis | Euclides; Lobachevsky, 1829; Poincaré, 1882; Gauss, 1827 (curvatura); Volder, 1959 (CORDIC); Bhaskara I, séc. VII (seno); Knuth, 1973 (dispersão multiplicativa); Penrose e Penrose, 1958; Penrose, 1992 (coomologia das figuras impossíveis); Huffman, 1971 e Clowes, 1971 (rotulação de arestas); Condorcet, 1785 (intransitividade); Möbius, 1858; Nitschke, 1966 (_ma_, o intervalo); o ensō da tradição zen (círculo de um traço) | `spatium` |
| L3 | Corpo | Esforço vital, marcha, postura, rocha | Espinosa, 1677 (_Ética_ III, prop. 6–7: conatus); cadência de marcha humana (Murray et al., 1964); _Daodejing_ 37 e 48 (não-ação); Kuleshov, 1929 (rosto neutro) | `corpus` |
| L4 | Escolha e consequência | Clinamen, crônicas, peso, serenidade, figuras | Lucrécio, II, 216–293; Camus, 1942; Nietzsche, 1882 (_A Gaia Ciência_ §341) e 1888 (_Ecce Homo_, amor fati); Heráclito, DK B12 e B91; Diógenes Laércio, VI; Epicuro (_Carta a Meneceu_) e Sexto Empírico (_Hipotiposes_ I) (ataraxia); _Bhagavad Gita_ 2.47 (ação sem direito ao fruto); _Samyutta Nikaya_ 12.21 (originação dependente); _Livro dos Mortos_, cap. 125 (pesagem do coração); _Liezi_, cap. 5 (o velho que remove montanhas); Eclesiastes 1:9 | `arbitrium` |
| L5 | Signo e forma | Tríade do signo; stili; linha orgânica; fonte; paleta; composição | Peirce, CP 2.247–249; Cícero, _De re publica_ II, 69 (concentus); Horta, 1893; Guimard, 1900; Beardsley, 1894; Mucha, 1896; Haeckel, 1899–1904; Bing, 1888 (Japonismo: Hokusai e Hiroshige como raiz da linha Art Nouveau); Bayer, 1973 (pontilhado ordenado); Bresenham, 1965; estética _wabi-sabi_ (imperfeição e erosão) e _kintsugi_ (reparo visível) | `concentus`, `signum` |
| L6 | Percepção | Codificação preditiva; cegueira à mudança; tensão; eventos raros; figura-fundo | Friston, 2010; Rensink, O'Regan e Clark, 1997; Fechner, 1860; Treisman e Gelade, 1980; Rubin, 1915; Zeigarnik, 1927; Schultz, Dayan e Montague, 1997; Posner e Cohen, 1984; Lavie, 1995; Kahneman, 2011; o kōan como pergunta sem resposta (Mumon, 1228, _Mumonkan_); _mono no aware_ (Motoori Norinaga, séc. XVIII) | `sensus` |
| L7 | Níveis de narração | Metalepse, autorreferência, oráculo, mise en abyme | Genette, 1972; Quintiliano, VIII.6.37 (transumptio); Hofstadter, 1979; Borges, 1940; Gödel, 1931; Kleene, 1938; von Foerster, 1974; Maturana e Varela, 1980; Foucault, 1966; Zhuangzi, cap. 2 (o sonho da borboleta); _Avatamsaka Sutra_ (a rede de Indra) | `transumptio` |
| L8 | Memória | Fechos de ciclo; ruínas; reconhecimento | Platão, _Mênon_ 81–86 (anamnese); Aristóteles, _Poética_ 11 (anagnórise); o _sankofa_ akan (voltar para buscar) | `memoria`, `arbitrium` |
| — | Limiar | Contrato entre o núcleo e o hospedeiro: facultates, indicia, tabula | Weiser e Brown, 1996 (computação calma) | `hospes` |
| — | Mundo | Composição dos níveis em um passo e um quadro | — | `mundus` |

## 2. Interfaces entre níveis

| Origem | Destino | Dado |
| --- | --- | --- |
| L0 | todos | `Verbum`, `Memoria` (leitura) |
| Limiar | L1 | `Indicia`: época, energia, sismo, automatismo, oclusão, partida, toque, deslocamento, inclinação, ociosidade |
| Limiar | consilium | `Facultates`: dimensões, profundidade de cor, cadência máxima, entradas, células, hemisfério, fuso, língua, orçamento |
| L1 | L3, L4 | fase circadiana, estação, fase lunar, festival, vento, ruído 1/f, temperatura |
| L2 | L5 | coordenadas projetadas, classe de estrato, rótulo de aresta |
| L3, L4 | L5 | cena: postura, fase da marcha, rocha, evento, ramo, crônica, figuras, ruínas |
| L6 | L0 | mutação em oclusão; anomalia liminar; porta de eventos raros; quadrante do próximo evento |
| L7 | L0 | oráculo: um bit de `modus` escrito pelo toque previsto |
| L8 | L5 | ruínas: contagem e posições; postura de sono após o reconhecimento |
| consilium | L5, Limiar | `Regimen`: stilus, estratos, intervalo, camadas, linhas de texto |

## 3. Glossário

Termos definidos. Grafia única. O nome no código é o termo em latim.

| Termo | Código | Definição | Fonte |
| --- | --- | --- | --- |
| Verbum | `Verbum` | Palavra de 64 bits que contém o estado presente do mundo: semen, conatus, onus, ascensus, curvatura, automatum, tranquillitas, modus | Leibniz, 1703; `04-arquitetura.md` §3 |
| Memoria | `Memoria` | Palavra de 64 bits que contém o passado e a intenção: clausurae, quadrans, eventus, ramus, gradus, oraculum | `04-arquitetura.md` §3 |
| Mundus | `Mundus` | Par (Verbum, Memoria) | — |
| Semen | `semen` | Estado de 16 bits do gerador de Galois; nunca zero; "sementes das coisas" | Lucrécio, I–II; Golomb, 1967 |
| Conatus | `conatus` | Esforço vital de 8 bits; decai com o atrito; regenera na quietude | Espinosa, _Ética_ III, prop. 6 |
| Onus | `onus` | Peso moral acumulado, 8 bits; altera a gravidade local, a forma da moldura e a paleta | _Livro dos Mortos_ 125; Espinosa, III (afetos) |
| Ascensus | `ascensus` | Progresso na encosta, 0..=255 | — |
| Curvatura | `curvatura` | Curvatura de Gauss do espaço, inteiro com sinal; 0 euclidiano; negativo hiperbólico; positivo elíptico | Gauss, 1827; Poincaré, 1882 |
| Automaton | `automatum` | Anel de 8 células sob a regra 30; gera encosta e vento | Wolfram, 1983 |
| Tranquillitas | `tranquillitas` | Serenidade de 6 bits; reduz a paleta, afina a linha, acelera a regeneração; exibida como hexagrama | Epicuro; Sexto Empírico; Sêneca, _De tranquillitate animi_; _Yijing_ via Leibniz, 1703 |
| Modus | `modus` | Modo epistemológico de 2 bits: vigilia, quies, somnium, amor fati | `03-especificacao.md` REQ-L4 |
| Vigilia | `Modus::Vigilia` | Modo cinético padrão | — |
| Quies | `Modus::Quies` | Modo de quietude: energia escassa; quadro estático bicolor | Lucrécio, III (quies) |
| Somnium | `Modus::Somnium` | Modo onírico: geometria não euclidiana sem aviso; revelado depois | Zhuangzi, cap. 2 |
| Amor fati | `Modus::AmorFati` | Modo de afirmação: a figura chuta a rocha por escolha; reinício voluntário | Nietzsche, 1888; Camus, 1942 |
| Clausurae | `clausurae` | Anel de 14 bits com o fecho dos ciclos: bit 0 = ciclo presente; bits 1..=13 = últimos 13 ciclos; 1 interrompido pelo observador, 0 concluído | — |
| Quadrans | `quadrans` | Último quadrante da tela tocado pelo observador, 0..=3 | — |
| Eventus | `eventus` | Identificador de 4 bits do clinamen presente | — |
| Ramus | `ramus` | Ramo escolhido no clinamen presente, 0..=2 | — |
| Gradus | `gradus` | Idade do evento presente em ticks, 10 bits, saturante (o clinamen do degrau rachado chama-se Rima) | — |
| Oraculum | `oraculum` | Cadeia de Markov de ordem 1 sobre 4 quadrantes, 16 células de 2 bits, com a qual a figura prevê o toque do observador | Markov, 1906; von Foerster, 1974 |
| Clinamen | `Clinamen` | Evento de desvio mínimo: micro-incidente com três ramos, em tempo e lugar determinados pelo semen | Lucrécio, II, 216–293 |
| Annales | `Annales` | Crônica: sequência de 4..=6 linhas com deslocamento em ticks que percorre a escada de registros, do íntimo ao epocal ou abstrato | _Samyutta Nikaya_ 12.21; Tácito |
| Persona | `Persona` | Figura que aparece sob condição de efeméride e estado: Diógenes, Heráclito, o daemon | Diógenes Laércio VI; Heráclito B12; Nietzsche §341 |
| Agnitio | `agnitio` | Reconhecimento: ao sair do somnium, a figura está adormecida na base; a ascensão não ocorreu | Aristóteles, _Poética_ 11 |
| Regimen | `Regimen` | Configuração escolhida por consilium: stilus, estratos, intervalo, camadas, linhas | Simon, 1955 |
| Stilus | `Stilus` | Idioma visual: Creta, Musivum, Littera, Bicolor | `05-estetica.md` |
| Creta | `Stilus::Creta` | Giz sobre ardósia: linhas de 1 px em 16 tons, preenchimento por pontilhado | — |
| Musivum | `Stilus::Musivum` | Mosaico: raster em baixa resolução com figuras de blocos; ampliação por vizinho mais próximo | — |
| Littera | `Stilus::Littera` | Letras: grade de células com glifos; a cena como texto | — |
| Bicolor | `Stilus::Bicolor` | Um bit por pixel pela matriz de Bayer 4×4 | Bayer, 1973 |
| Concentus | `Concentus` | Acorde: vetor único de afinação dos canais (tom, contraste, espessura de linha, cadência, registro do texto, curvatura, índice sonoro) derivado do estado, da efeméride e do regimen; todo meio lê o concentus, nenhum meio lê o estado diretamente para decidir forma | Cícero, _De re publica_ II, 69 |
| Registro | `registrum` | Escala do texto da crônica: íntimo (segundos), vivo (minutos), civil (anos), epocal (séculos), abstrato (ideias) | `07-narrativa.md` |
| Stratum | `stratum` | Classe de profundidade de 2 bits (0, 1, 2; 3 = vazio) com dominância cíclica | Condorcet, 1785; Penrose, 1992 |
| Facultates | `Facultates` | Capacidades declaradas pelo hospedeiro | — |
| Indicia | `Indicia` | Medidas do meio fornecidas pelo hospedeiro a cada tick | — |
| Tabula | `Tabula` | Superfície de saída raster: pixels de 8 bits fornecidos pelo hospedeiro | — |
| Cellae | `Cellae` | Superfície de saída em células: glifo, tom e estrato por célula, para o stilus Littera | — |
| Nuntius | `Nuntius` | Conjunto de bits devolvido por `step` que anuncia marcos: fecho, clinamen, persona, agnitio, metalepse, evento raro, lasca | — |
| Lingua | `Lingua` | Língua do texto da experiência: inglês ou português | `convencoes.md` §1 |
| Hospes | `hospes` | Hospedeiro: programa que mede o meio, chama o núcleo e exibe a tabula | — |
| Tick | — | Uma chamada de `step` | — |
| Festum | `festum` | Dia astronômico: solstício, equinócio, lua nova, lua cheia | — |
| Sismo | `seism` | Agitação externa 0..=255: acelerômetro, entropia de interação, carga | — |
| Automatismo | `automatism` | Grau 0..=255 de regularidade mecânica da interação, medido pelo hospedeiro | Kahneman, 2011 |
| Ma | — | Intervalo: o vazio entre nós e traços como elemento da composição | Nitschke, 1966 |
| Ensō | — | Círculo de um traço, imperfeito, desenhado em giz pela figura quando o oráculo prevê o toque | tradição zen |
| Kintsugi | — | Reparo visível: a lasca da rocha é unida por linha de tom máximo no modo amor fati | estética japonesa |
