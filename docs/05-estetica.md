# 05 — Estética

## 1. Princípio

A linha orgânica e dinâmica é o princípio comum a todos os stili. A fonte histórica é a Art
Nouveau (Horta, 1893; Guimard, 1900; Beardsley, 1894; Mucha, 1896) e a sua raiz documentada no
Japonismo (Bing, 1888; Hokusai; Hiroshige): curva de chicote, assimetria, ornamento que é
estrutura, planos chapados sem gradiente, linha de espessura variável. O princípio é adotado; o
estilo de época não. A forma final de cada quadro é escolhida por `consilium` segundo a situação
narrativa e os requisitos não funcionais.

## 2. Material

Giz sobre ardósia: fundo escuro, traço claro, 16 tons, sem antisserrilhamento, sem gradiente além
do pontilhado ordenado. O vazio entre traços é elemento da composição (intervalo, _ma_). A erosão
do traço com o tempo e o reparo visível da rocha pertencem à mesma doutrina (_wabi-sabi_,
_kintsugi_).

## 3. Stili

| Stilus | Forma | Situação que o escolhe | Significado |
| --- | --- | --- | --- |
| Creta | Linhas de 1 px (× escala) em 16 tons; preenchimento por pontilhado; curvas de chicote na moldura | Vigilia com energia alta; qualquer resolução | O mundo desenhado à mão, apagável |
| Musivum | Raster em baixa resolução ampliado por vizinho mais próximo; figuras de blocos | Energia média; hospedeiros de pouca computação; Somnium em transição | O mundo discreto, contado em tesselas |
| Littera | Grade de células; a cena vira texto pela rampa ` .:-=+*#%@`; as crônicas permanecem legíveis | Hospedeiros só de texto; redução eidética (energia 39..=76 com `tranquillitas` ≥ 48) | O mundo reduzido aos seus signos |
| Bicolor | Um bit por pixel pela matriz de Bayer 4×4; tom ≤ 1 → 0 e tom ≥ 13 → 15, pontilhado entre eles | Quies; painéis de 1 bit; `color_bits` = 1 | O mundo em repouso: presença e ausência |

Tom e classe de estrato de cada elemento no quadro de giz, base dos demais stili (REQ-L5-03). As
linhas e os glifos usam o tom do elemento, conforme a tabela. Encosta, crista e ruínas
respeitam adicionalmente o teto 15−`contrastus`, inclusive quando traçadas por colunas
sólidas; a sombra usa tom 2. O maior contraste é 13 = 15−2: tom 2 é o menor tom que
Bicolor conserva por pontilhado, pois tons 0 e 1 viram preto. Em Quies, a ardósia de tom 1
vira preto e a encosta limitada a tom 2 conserva pontos brancos.

| Elemento | Tom | Classe | Traçado |
| --- | --- | --- | --- |
| Ardósia | 1 | 3 | fundo |
| Estrelas da constelação | 6 | 2 | pontos |
| Arestas do grafo | 9 | 2 | linhas de s px |
| Nós do grafo | 14 | 2 | quadrados de 2s×2s |
| Aforismo | 12 sobre ardósia 1 | 2 | células 6×10·s, respiro s; âncora ajustada apenas para preservar hexagrama, lua e contenção no quadro (REQ-L5-06) |
| Hexagrama | 8 | 2 | linhas de s px |
| Lua | 12, pontilhada em 9 | 2 | disco menos o disco de sombra |
| Encosta | 3..=6, do estrato mais distante ao mais próximo | 1 no mais distante com 2 ou mais estratos; 2 nos demais | pontilhado (camada de pontilhado) ou colunas sólidas |
| Crista da encosta | 6..=9 (tom do estrato + 3) | a do estrato | um pixel por coluna |
| Ruínas | 7 | 2 | linhas de s px |
| Moldura | 9 | 0 | linhas de `linea`·s px |
| Sombra da figura | 2 | 1 | linha de s px |
| Figura | 15 | 1 | linhas de `linea`·s px e bloco da cabeça |
| Rocha | 12 (bit 1) e 4 (bit 0); contorno 15 | 1 | disco com a textura; contorno depois do interior |
| Crônicas | 13 | 0 | glifos |
| `Verbum` após agnitio | 10 | 0 | glifos |
| Células de Littera no raster | fundo 1; glifo da rampa 12 | fundo 3; glifo 1 | célula de 6×10·s |

## 4. Composição

| Elemento | Regra | Constante |
| --- | --- | --- |
| Escala inteira | s = max(1, min(W/320, H/180)) | — |
| Margem | m = ⌊W/(8·δS)⌋ | δS = 1 + √2 = `0x26A0A` |
| Plinto das crônicas | altura P = m + `lines`·10·s na base; a montanha assenta sobre o texto | — |
| Cena | altura S = H − P | — |
| Encosta | de (W·φ⁻², S) a (W − m, S·φ⁻²) | φ⁻² = `0x61C9` |
| Região do céu | x ∈ [m, W·φ⁻¹]; y ∈ [m, S·φ⁻²] | φ⁻¹ = `0x9E37` |
| Região das crônicas | x ∈ [m, W − m]; y ∈ [S, H − m]; 48 glifos por linha a 320 px | — |
| Recuo das linhas | a linha i recua min(3·s·i, 6·s·(glifos por linha − 40)) px, nada com 40 glifos por linha ou menos; cada linha conserva ≥ 40 glifos | — |
| Altura da figura | S·φ⁻³ (24 px a 320×180) | φ⁻³ = `0x3C6F` |
| Diâmetro da rocha | ⌈altura da figura / φ⌉; textura 8×8 ampliada por max(1, ⌊(diâmetro + 4)/8⌋) | φ⁻¹ = `0x9E37` |
| Estratos | 4 faixas de paralaxe com velocidades φ⁻¹..φ⁻⁴ | — |
| Harmonia corrompida | com `onus` > 200, a borda direita do céu em ⌊W·(φ⁻¹ ⊕ `0x1337`)⌋ (REQ-L5-12) | — |

## 5. Linha

- Espessura: s quando `tranquillitas` ≥ 32; 2·s abaixo (linha grossa e angulosa sob agitação).
- Curva de chicote: soma de 3 harmônicos no eixo horizontal com fases por `semen` (REQ-L5-05);
  amplitudes murcham com `onus` e com energia baixa; a moldura sobe dos cantos inferiores à margem
  superior; o lado direito desloca o harmônico k em k quartos de volta, o que faz a assimetria
  entre os lados.
- Rótulo de aresta (convexa, côncava, oclusiva) muda ao longo de uma mesma reta nas figuras
  impossíveis; nas demais linhas, permanece constante.

## 6. Figura

- Altura S·φ⁻³: o esqueleto é dado em unidades de h/24, levadas a pixels por round(k·h/24); a
  cabeça é um bloco liso de lado max(1, round(4·h/24)) que começa na unidade 23 (rosto de 4×4 px
  sem traços com h = 24, a 320×180); a expressão vem da postura, da cadência e da luz.
- Silhueta legível em todos os stili em menos de 1 s: a figura e a rocha têm contraste mínimo de 8
  tons com o fundo.
- Posturas: ereta, curvada, ajoelhada, contemplativa, adormecida, voltada ao observador.
- A rocha é a leitura direta do `Verbum`; a lasca retira um bit. As duas linhas de `semen`
  usam tons 7/5; as demais usam 12/4. Em contemplação, células de escala ≥ 8 contêm a
  microtextura: 15/12 no macro aceso, 6/4 no apagado, com precedência sobre os tons de `semen`.
- Em AmorFati, a rocha recebe a costura kintsugi de tom 15 e espessura s. Quatro pontos em
  cx−r, cx−r/3, cx+r/3 e cx+r têm desvios verticais derivados dos quatro bytes de
  `mix(Verbum ≫ 48)`: byte mod (r/2+1)−r/4. Três segmentos unem esses pontos.
- Com PERSONAE, Diógenes aparece ereto em `ascensus`+40 (saturado em 255), tom 13. A lâmpada
  é um disco de raio s e tom 15 em (cabeça.x+lado+2s, cabeça.y+2·lado).
- Heráclito aparece contemplativo em `ascensus`−48 (saturado em 0), tom 11. A pira no pé
  da encosta é um triângulo de altura h/2 e tom 14; seu ápice desloca s nas fases ímpares da marcha.
- O daemon aparece ereto no cume (`ascensus` = 255), tom 4. Desligar PERSONAE oculta a
  figura secundária e seu atributo; a contemplação da figura principal continua sendo lei do estado.

## 7. Cor

Paleta de 16 tons entre ardósia (18, 20, 24) e giz (240, 240, 245). `tranquillitas` reduz o croma;
`onus` > 200 desloca para sépia; Somnium desloca para azul frio; AmorFati coloca ouro no tom 15;
`festum` eleva o tom 15 em 8 %. Nenhuma outra cor.

## 8. Movimento

- Marcha a 64 passos/min (48 com `conatus` < 48), com a taxa modulada em ±12 % por quatro senos de
  períodos 7,5 s, 15 s, 30 s e 60 s e amplitude proporcional ao período (espectro 1/f); a fase nunca
  recua (REQ-L3-01).
- Cadência por regimen: 24 Hz (vigilia), 8 Hz (acedia), 4 Hz (estroboscopia), 1/60 Hz (quies).
- Nada entra em cena sob o olhar: personae entram com o observador ausente há 60 s ou em
  oclusão (REQ-L4-09); ruínas nascem no fecho do ciclo (REQ-L4-07, L4-08); mudanças de
  topologia ocorrem em oclusão (REQ-L6-01) ou por deriva abaixo do limiar diferencial.

## 9. Mudança de stilus por situação

| Situação | Stilus forçado ou preferido |
| --- | --- |
| Quies | Bicolor, 1 quadro/min |
| Somnium | Creta ou Musivum com curvatura e `strata` ≥ 2; Littera só em hospedeiro de células |
| Redução eidética (energia 39..=76 ∧ `tranquillitas` ≥ 48) | Littera preferido |
| AmorFati | Creta preferido (bônus de valor); ouro no tom 15 em qualquer stilus |
| `onus` > 200 | qualquer stilus com harmonia corrompida e espaço negativo ativo |

## 10. Concentus (o acorde)

A integridade conceitual atravessa os meios por um único vetor, `Concentus`, calculado antes de
qualquer desenho e lido por todos os canais. Nenhum canal decide sozinho.

| Canal | Campo do concentus | Derivação |
| --- | --- | --- |
| Tom | `tonus` 0..=15 e paleta | `onus`, `tranquillitas`, `modus`, `festum` (REQ-L5-02) |
| Contraste | `contrastus` 8..=13; tom de cada traço da encosta e das ruínas ≤ 15−`contrastus` | energia 179..=255 → 8; 77..=178 → 10; 39..=76 → 12; 0..=38 → 13; Quies força 13 = 15−2 |
| Delimitação | `linea` ∈ {s, 2s} e rótulo de aresta padrão | `tranquillitas` (REQ-L5-05); impossibilia por condição |
| Cadência | `cadentia`: cadência base em passos/min (64, 48 ou 0) | regimen (`tick_ms`); `conatus` |
| Registro do texto | `registrum` ∈ {íntimo, vivo, civil, epocal, abstrato} | última linha do prefixo efetivo de `annales`, limitado por ANNALES, `regimen.lines` e, em células, `rows`; Intimum sem linhas |
| Curvatura | `curvatura` aplicada | `curvatura` do `Verbum` e `modus` |
| Som (índice; sem síntese na 0.1) | `sonus` = ⌊2040/`isqrt(64+onus)`⌋: 255 com `onus` 0, 120 com `onus` 255, não crescente | massa = `onus` + 64 (rocha); 2040 = 255·√64; divisão inteira pela raiz inteira inferior |
| Línguas | `lingua` e forma dos aforismos (□, ◊) | `Facultates.lingua`; nó de ramificação |

O concentus é o mesmo para todos os stili sob as mesmas capacidades e situação: um quadro em Littera e um quadro
em Creta do mesmo instante têm o mesmo tom, o mesmo contraste, o mesmo registro e o mesmo índice
sonoro. Na saída em células, a grade real normaliza as capacidades antes do cálculo: uma
grade com menos linhas pode mudar o registro porque mostra um prefixo menor. O contraste
é aplicado ao fundo antes das conversões de stilus; em Bicolor, o limiar de Bayer preserva
a ardósia preta e a encosta pontilhada de Quies. O teto 15−13 conserva o tom 2, primeiro
tom que Bicolor admite acima do preto.

## 11. Proibições

Gradientes contínuos; antisserrilhamento; texto que explique; números de estado; logotipos;
cores fora da paleta; animação que acelere com a interação; qualquer elemento sem entrada em
`03-especificacao.md`.
