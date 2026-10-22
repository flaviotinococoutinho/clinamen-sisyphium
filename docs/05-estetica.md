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
| Bicolor | Um bit por pixel pela matriz de Bayer 4×4 | Quies; painéis de 1 bit; `color_bits` = 1 | O mundo em repouso: presença e ausência |

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
| Altura da figura | S·φ⁻⁴ | φ⁻⁴ = `0x255A` |
| Diâmetro da rocha | altura da figura / φ (textura 8×8 ampliada) | φ = `0x19E37` |
| Estratos | 4 faixas de paralaxe com velocidades φ⁻¹..φ⁻⁴ | — |
| Harmonia corrompida | com `onus` > 200, φ⁻¹ ⊕ `0x1337` nas proporções do céu (REQ-L5-12) | — |

## 5. Linha

- Espessura: s quando `tranquillitas` ≥ 32; 2·s abaixo (linha grossa e angulosa sob agitação).
- Curva de chicote: soma de 3 harmônicos; amplitudes murcham com `onus` e com energia baixa; a
  moldura cresce dos cantos inferiores; assimetria entre esquerda e direita por `semen`.
- Rótulo de aresta (convexa, côncava, oclusiva) muda ao longo de uma mesma reta nas figuras
  impossíveis; nas demais linhas, permanece constante.

## 6. Figura

- Altura H·φ⁻⁴; rosto de 4×4 px sem traços; a expressão vem da postura, da cadência e da luz.
- Silhueta legível em todos os stili em menos de 1 s: a figura e a rocha têm contraste mínimo de 8
  tons com o fundo.
- Posturas: ereta, curvada, ajoelhada, contemplativa, adormecida, voltada ao observador.
- A rocha é a leitura direta do `Verbum`; a lasca retira um bit; em AmorFati a lasca recebe uma
  linha de tom 15 (reparo visível).

## 7. Cor

Paleta de 16 tons entre ardósia (18, 20, 24) e giz (240, 240, 245). `tranquillitas` reduz o croma;
`onus` > 200 desloca para sépia; Somnium desloca para azul frio; AmorFati coloca ouro no tom 15;
`festum` eleva o tom 15 em 8 %. Nenhuma outra cor.

## 8. Movimento

- Marcha a 64 passos/min modulada por ruído 1/f; respiração visível no torso pela mesma fonte.
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
| Contraste | `contrastus` 8..=15 (diferença mínima figura–fundo) | energia (menor energia, maior contraste), `modus` Quies = 15 |
| Delimitação | `linea` ∈ {s, 2s} e rótulo de aresta padrão | `tranquillitas` (REQ-L5-05); impossibilia por condição |
| Cadência | `cadentia` em passos/min e intervalo | regimen; ruído 1/f; `conatus` |
| Registro do texto | `registrum` ∈ {íntimo, vivo, civil, epocal, abstrato} | linha da crônica revelada; persona; agnitio |
| Curvatura | `curvatura` aplicada | `curvatura` do `Verbum` e `modus` |
| Som (índice; sem síntese na 0.1) | `sonus`: altura ∝ 1/√massa, ressonância ∝ massa, respiração 60..=70/min, clique por linha | massa = `onus` + rocha; cadência; crônicas |
| Línguas | `lingua` e forma dos aforismos (□, ◊) | `Facultates.lingua`; nó de ramificação |

O concentus é o mesmo para todos os stili na mesma situação: um quadro em Littera e um quadro
em Creta do mesmo instante têm o mesmo tom, o mesmo contraste, o mesmo registro e o mesmo índice
sonoro. A sinergia da obra é verificável nesse ponto.

## 11. Proibições

Gradientes contínuos; antisserrilhamento; texto que explique; números de estado; logotipos;
cores fora da paleta; animação que acelere com a interação; qualquer elemento sem entrada em
`03-especificacao.md`.
