# 03 — Especificação

Requisitos numerados. Verbos normativos com o sentido da RFC 2119. Métodos de verificação: U =
teste unitário nomeado pela lei; P = teste de propriedade com casos gerados pelo gerador de Galois;
G = quadro-ouro (hash FNV-1a de 64 bits do raster); I = inspeção ou varredura automatizada; M =
medição registrada no PR. A coluna "Módulo" liga o requisito ao código; a coluna "Teste" liga ao
nome do teste. Versão 0.1 salvo indicação.

## L0 — Substrato

| ID | Requisito | Módulo | Verificação |
| --- | --- | --- | --- |
| REQ-L0-01 | O `Verbum` DEVE ter o leiaute: bits 0..=15 `semen`; 16..=23 `conatus`; 24..=31 `onus`; 32..=39 `ascensus`; 40..=47 `curvatura` (inteiro com sinal); 48..=55 `automatum`; 56..=61 `tranquillitas`; 62..=63 `modus`. Empacotar e desempacotar DEVEM ser inversos para todo padrão de 64 bits com `semen` ≠ 0. | `verbum` | P `verbum_pack_unpack_identity` |
| REQ-L0-02 | A `Memoria` DEVE ter o leiaute: bits 0..=13 `clausurae`; 14..=15 `quadrans`; 16..=19 `eventus`; 20..=21 `ramus`; 22..=31 `gradus`; 32..=63 `oraculum`. Empacotar e desempacotar DEVEM ser inversos para todo padrão de 64 bits. | `memoria` | P `memoria_pack_unpack_identity` |
| REQ-L0-03 | O estado persistido DEVE ser o texto de 32 dígitos hexadecimais minúsculos: `Verbum` (16 dígitos, nibble mais significativo primeiro) seguido de `Memoria` (16 dígitos). A leitura DEVE aceitar maiúsculas, DEVE rejeitar qualquer outro comprimento ou caractere e DEVE normalizar `semen` = 0 para `0xACE1`. Para todo mundo válido, ler(escrever(m)) = m. | `mundus` | P `format_parse_roundtrip`; U `parse_rejects_invalid` |
| REQ-L0-04 | `Mundus::genesis(seed)` DEVE produzir: `semen` = 16 bits baixos de `seed` ou `0xACE1` se zero; `conatus` 192; `onus` 32; `ascensus` 0; `curvatura` 0; `automatum` `0x10`; `tranquillitas` 24; `modus` Vigilia; `clausurae` 0; `quadrans` 0; `eventus` = bits 16..=18 de `seed`; `ramus` 3 (aguardando); `gradus` 0; `oraculum` `0x5555_5555`. | `mundus` | U `genesis_fields` |
| REQ-L0-05 | `step` DEVE ser total: para todo mundo válido, todo `Indicia` e todo `Regimen`, o resultado é um mundo válido e nenhuma operação entra em pânico. | `mundus` | P `step_is_total` (65 535 sementes × 8 indicia) |
| REQ-L0-06 | O núcleo NÃO DEVE conter ponto flutuante, alocação, dependências, `unsafe` ou acesso a fatia sem verificação. | todos | I `cargo clippy` com `float_arithmetic`, `indexing_slicing`, `unwrap_used`, `expect_used`, `panic`, `arithmetic_side_effects` proibidos e `abs` de inteiros com sinal vetado (`disallowed_methods` em `clippy.toml`); U `primitives_total_at_extremes` |

## L1 — Natureza e tempo

| ID | Requisito | Módulo | Verificação |
| --- | --- | --- | --- |
| REQ-L1-01 | O gerador DEVE ser um registrador de deslocamento de Galois de 16 bits com polinômio de retroalimentação `0xB400` (x¹⁶ + x¹⁴ + x¹³ + x¹¹ + 1). Para toda semente ≠ 0 o período DEVE ser 65 535. | `natura` | U `galois_full_period` |
| REQ-L1-02 | O autômato DEVE ser um anel de 8 células sob a regra 30: célula' = esquerda ⊕ (célula ∨ direita). | `natura` | U `rule30_known_sequence` |
| REQ-L1-03 | O ruído 1/f DEVE ser a média de 8 oitavas, em que a oitava k vale `hash(k, t ≫ k) mod 256` para o tempo t em unidades de 1/32 s; a oitava k troca de valor exatamente a cada 2ᵏ unidades. Saída em 0..=255. | `natura` | U `pink_octave_rates`; U `pink_mean_bounds` |
| REQ-L1-04 | O mapa logístico x' = r·x·(1 − x) DEVE ser calculado em Q16.16. O limiar de Feigenbaum r∞ DEVE valer `0x391E8` (3,56995). O parâmetro r DEVE ser 2,5 + 0,6·noturno + 0,6·min(ociosidade/3600 s, 1). A elegibilidade do somnium é r ≥ r∞. | `natura`, `arbitrium` | U `logistic_fixed_point`; U `somnium_eligibility_threshold` |
| REQ-L1-05 | O atrator de Hénon (a = 1,4 → `0x16666`; b = 0,3 → `0x4CCD`) em Q16.16 DEVE permanecer limitado a \|x\| ≤ 1,5 e \|y\| ≤ 0,5 por 4 096 iterações a partir de (0, 0). | `natura` | U `henon_bounded` |
| REQ-L1-06 | A efeméride DEVE derivar de `epoch_ms`, do fuso em minutos e de `australis`: fase circadiana 0..=255 (0 = meia-noite local; noturno fora de 64..=191); fase da estação 0..=255 (0 = equinócio de março, dia 79; invertida no sul); fase lunar 0..=255 (mês sinódico 2 551 443 s a partir da época 947 182 440 s); `festum` quando o dia do ano está a ≤ 1 dia de 79, 172, 265 ou 355, ou a fase lunar está a ≤ 4 de 0 ou 128. | `ephemeris` | U `ephemeris_known_dates` (solstício 2026-06-21; lua nova 2000-01-06) |
| REQ-L1-07 | A cadência do mundo (`tick_ms`) por faixa de energia DEVE ser: ≥ 179 → 42 ms; 77..=178 → 125 ms; 39..=76 → 250 ms; < 39 → 60 000 ms com stilus Bicolor. `consilium` PODE alongar `interval_ms` acima de `tick_ms`; NÃO DEVE encurtá-lo abaixo de 1000/`max_hz`. | `consilium` | U `regimen_energy_bands` |
| REQ-L1-08 | O vento DEVE ser popcount(`automatum`) ≥ 6. Com vento, o limiar de passo (REQ-L4-05) é dividido por 2. | `natura`, `arbitrium` | U `wind_halves_pace` |

## L2 — Espaço

| ID | Requisito | Módulo | Verificação |
| --- | --- | --- | --- |
| REQ-L2-01 | A aritmética de ponto fixo DEVE ser Q16.16 em `i32` com produto e quociente intermediários em `i64`, saturantes. | `spatium` | P `fixed_mul_div_saturate` |
| REQ-L2-02 | O seno DEVE usar a fórmula de Bhaskara I sobre o ângulo em 0..=65 535 (volta completa; meia-volta = 32 768). O erro absoluto máximo DEVE ser ≤ 110 unidades de Q16.16 (0,0017). | `spatium` | U `bhaskara_max_error` |
| REQ-L2-03 | A raiz quadrada inteira DEVE devolver ⌊√n⌋ para todo `u32`. | `spatium` | P `isqrt_floor` |
| REQ-L2-04 | A rotação CORDIC com 16 iterações e compensação de ganho DEVE rotacionar vetores unitários com erro ≤ 4 unidades de Q16.16 por componente. | `spatium` | U `cordic_unit_rotation` |
| REQ-L2-05 | A dispersão multiplicativa DEVE ser `(k · 0x9E37_79B9) ≫ (32 − b)` para b bits de saída; em 64 bits, `0x9E37_79B9_7F4A_7C15`. | `spatium` | U `knuth_constants` |
| REQ-L2-06 | A projeção de curvatura DEVE levar o deslocamento (dx, dy) em pixels de um ponto ao centro da tabula (⌊W/2⌋, ⌊H/2⌋) ao deslocamento (round(dx·f/65 536), round(dy·f/65 536)), com arredondamento simétrico (meio afastado do zero) e o fator radial f em Q16.16 calculado a partir do quadrado do raio normalizado r2 = (dx² + dy²)·65 536/metade² (Q16.16; metade = max(1, ⌊max(W, H)/2⌋); `i64` saturante), limitado a 131 072 (r² ≤ 2, o canto de um quadro quadrado; além dele o fator é constante e o mapa segue radial e linear): f = 65 536 + (`curvatura`·r2) ≫ 7, isto é 1 + κ·r², com κ = `curvatura`/128 ≥ 0; f = 2³²/(65 536 + (\|`curvatura`\|·r2) ≫ 7), isto é 1/(1 + \|κ\|·r²), com κ < 0. κ = 0 é identidade. κ < 0 contrai de modo monótono enquanto \|κ\|·r² ≤ 1, o que vale em todo o domínio operacional (\|`curvatura`\| ≤ 64), e coincide com 1 + κ·r² até a segunda ordem (aproximação do disco de Poincaré). κ > 0 expande, e cada ponto DEVE ser escrito como um bloco de lado b = 1 + ⌈3·`curvatura`·r2/2²³⌉ (a derivada radial 1 + 3κr²), limitado a 8, com o canto superior esquerdo no ponto projetado. Em coordenadas normalizadas em Q8.8 (256 = metade), a mesma lei vale com r2 = x² + y². | `spatium`, `signum` | U `curvature_identity_at_zero`; U `curvature_sign_direction`; U `curvature_monotone`; U `curvature_factor_law`; U `curvature_block_law`; U `curvature_density` |
| REQ-L2-07 | Os estratos de paralaxe DEVEM ter velocidades φ⁻¹ `0x9E37`, φ⁻² `0x61C9`, φ⁻³ `0x3C6F`, φ⁻⁴ `0x255A` (Q16.16). | `spatium` | U `strata_velocities_phi` |
| REQ-L2-08 | A dominância entre classes de estrato DEVE ser cíclica: 0 ≻ 2, 2 ≻ 1, 1 ≻ 0; a classe 3 (vazio) é sempre sobrescrita; classe igual aplica ⊕ na luminância. A tabela empacotada DEVE ser `0x0A9C` com índice (nova ≪ 2) \| atual. As classes por camada seguem `04-arquitetura.md` §5. | `spatium`, `signum` | U `stratum_dominance_cycle`; U `stratum_lut_matches_rule`; U `composition_classes` |
| REQ-L2-09 | A projeção dimétrica 2:1 DEVE ser u = x − z, v = y + ⌊(x + z)/2⌋; o vetor (2, −2, 2) DEVE projetar-se no zero. | `spatium` | P `isometric_kernel` |
| REQ-L2-10 | A escada de Penrose com 4 lances de `span` passos DEVE satisfazer posição(t + 4·span) = posição(t) para a tripla (x, y, classe), com classe 0 no primeiro lance e 2 no último para todo `span`, de modo que o primeiro lance domine o quarto. | `spatium` | U `penrose_closed_loop` |
| REQ-L2-11 | A composição DEVE derivar de φ e δS: margem m = ⌊W/(8·δS)⌋; escala inteira s = max(1, min(W/320, H/180)); plinto das crônicas de altura P = m + `lines`·10·s na base; altura de cena S = H − P; região do céu x ∈ [m, W·φ⁻¹], y ∈ [m, S·φ⁻²]; encosta de (W·φ⁻², S) a (W − m, S·φ⁻²); região das crônicas x ∈ [m, W − m], y ∈ [S, H − m], com ⌊(W − 2m)/(6·s)⌋ glifos por linha (48 a 320 px); largura lógica mínima 272 px; altura da figura S·φ⁻³; diâmetro da rocha = ⌈altura da figura/φ⌉ com a textura 8×8 ampliada por max(1, ⌊(diâmetro + 4)/8⌋). Exato para todo `u16`. | `spatium`, `corpus` | U `composition_constants`; U `figure_fits_box` |

## L3 — Corpo

| ID | Requisito | Módulo | Verificação |
| --- | --- | --- | --- |
| REQ-L3-01 | A fase da marcha DEVE ser ⌊quadros(t)⌋ mod 4, com quadros(t) = 4·c·t/60 000 + Σₖ Aₖ·sen(2π·t/Pₖ + k·π/2), t = `epoch_ms` mod 60 000, c a cadência base do `Concentus` em passos/min (64; 48 com `conatus` < 48; 0 com `tick_ms` ≥ 1 000), Pₖ = 7 500·2ᵏ ms (k = 0..=3) e Aₖ = 0,03·(4c/60 000)·Pₖ/(2π) quadros: cada termo contribui ±3 % da taxa base (±12 % no total; amplitude proporcional ao período, espectro 1/f), a contagem nunca decresce e cada minuto avança 4c quadros, 4 por passo. Em inteiros: quadros em Q16.16, seno de Bhaskara (REQ-L2-02) e Aₖ = 2ᵏ·c·156 em Q16.16 (0,03·(4/60 000)·7 500/(2π)·65 536 = 156,46). | `corpus`, `concentus` | U `gait_cadence`; U `gait_monotone`; U `cadentia_base` |
| REQ-L3-02 | A postura DEVE ser função pura do estado, nesta precedência: voltada ao observador quando `leaving`; adormecida por 64 ticks após agnitio (`eventus` = 12 ∧ `gradus` < 64), deitada com a cabeça morro acima, contra a rocha; contemplativa em Quies ou durante persona (`eventus` ∈ 13..=15 ∧ `ramus` = 0 ∧ `gradus` < 240); ajoelhada se `conatus` < 48; curvada se `onus` ≥ 128; ereta nos demais casos. | `corpus`, `signum` | U `posture_table`; U `posture_reads_the_persona`; U `dormiens_head_toward_boulder` |
| REQ-L3-03 | A textura da rocha DEVE ser a leitura direta dos 64 bits do `Verbum` em 8×8, com rotação de rolamento por deslocamento circular de 8·k bits, k = `ascensus` mod 8. | `corpus` | U `boulder_is_verbum` |
| REQ-L3-04 | Uma lasca DEVE ocorrer quando `seism` ≥ 240 ∧ (`semen` ∧ 0x3F) = 0: inverte um bit do `Verbum` em 48..=61 escolhido por `semen` ≫ 6. A lasca NÃO DEVE zerar `semen`. | `corpus` | U `chip_flips_one_bit` |
| REQ-L3-05 | O rosto DEVE ter 4×4 px sem traços. | `corpus` | I `sprite_face_blank` |

## L4 — Escolha e consequência

| ID | Requisito | Módulo | Verificação |
| --- | --- | --- | --- |
| REQ-L4-01 | Os clinamina DEVEM ocupar `eventus` 0..=7 (8 na versão 0.1), cada um com faixa de lugar (`ascensus`), micro-incidente e 3 rami. Reservados: 11 metalepse; 12 agnitio; 13 Diógenes; 14 Heráclito; 15 daemon. | `arbitrium` | U `event_table_complete` |
| REQ-L4-02 | Um clinamen DEVE disparar somente quando `ramus` = 3 ∧ `ascensus` está na faixa do `eventus` ∧ (`semen` ∧ 0xFF) < 24. Ao disparar: `ramus` = (`semen` ≫ 12) mod 3; `gradus` = 0; aplica Δonus, Δconatus, Δtranquillitas do ramus. | `arbitrium` | U `clinamen_fires_in_band`; U `clinamen_applies_deltas` |
| REQ-L4-03 | Um clinamen DEVE encerrar quando `gradus` ≥ 600: `eventus` = (`eventus` + 1 + (`semen` ≫ 8) mod 7) mod 8; `ramus` = 3; `gradus` = 0. | `arbitrium` | U `clinamen_closes_and_rotates` |
| REQ-L4-04 | Cada ramus DEVE ter 4..=6 linhas de crônica com deslocamentos crescentes em ticks (24, 72, 144, 240, 360, 480); a linha i é visível quando `gradus` ≥ deslocamento_i; cada linha tem no máximo 40 glifos. | `arbitrium` | U `annales_reveal_offsets` |
| REQ-L4-05 | O passo DEVE ocorrer quando (`semen` ∧ 0x7FFF) < T, com T = clamp(`conatus` − `onus`/2, 16, 255), dividido por 2 com vento. Em `ascensus` ≥ 240, T é dividido por 2^((ascensus − 240)/4); o passo final para 255 exige `conatus` ≥ 160 ∧ sem vento. | `arbitrium` | U `pace_threshold`; U `zeigarnik_asymptote` |
| REQ-L4-06 | Leis contínuas por tick: `conatus` −1 no tick em que ocorre um passo quando (`semen` ∧ 3) = 0; `conatus` +1 num tick sem passo quando (`semen` ∧ 0x1FF) = 0 em subida ou (`semen` ∧ 0x1F) = 0 em contemplação (Quies, persona, sono); `conatus` −1 quando `seism` ≥ 128 ∧ (`semen` ∧ 3) = 0; `onus` −1 em direção a 32 quando (`semen` ∧ 0x3F) = 0; `tranquillitas` +1 quando `idle_s` ≥ 600 ∧ (`semen` ∧ 0xFF) = 0, −1 por tick com `seism` ≥ 64. Tudo saturante nos domínios. | `arbitrium` | U `continuous_laws` |
| REQ-L4-07 | Fecho de ciclo em `ascensus` = 255: `clausurae` ≪ 1 dentro de 14 bits (bit 0 novo = 0); `ascensus` = 0; `onus` −8; `conatus` +24; se `modus` = AmorFati, `tranquillitas` +4 e `modus` = Vigilia. | `arbitrium` | U `cycle_closure` |
| REQ-L4-08 | Interrupção (`leaving`): bit 0 de `clausurae` = 1; a cena mostra a figura voltada ao observador e uma linha de interrupção escolhida por hash(`clausurae`, `eventus`). As ruínas são popcount(bits 1..=13 de `clausurae`). | `arbitrium`, `signum` | U `interruption_marks_cycle`; U `ruins_count` |
| REQ-L4-09 | Personae disparam somente com `idle_s` ≥ 60 ou `occluded` (nada entra em cena sob o olhar): Diógenes com fase circadiana 112..=144 ∧ `ascensus` 96..=160 ∧ `ramus` = 3 ∧ (`semen` ∧ 0x1FF) = 0 → `ascensus` −32, `onus` +8; Heráclito com fase 184..=200 ∧ `ascensus` 64..=128 ∧ mesma porta → `onus` −16, `curvatura` +8; daemon com fase < 32 ∧ `ascensus` ≥ 250 ∧ `idle_s` ≥ 1800 → se `conatus` ≥ 200, `modus` = AmorFati, senão `onus` +16. A persona ocupa `eventus` 13..=15 com `ramus` 0 e dura até `gradus` ≥ 240. | `arbitrium` | U `personae_conditions` |
| REQ-L4-10 | Lei do modus por tick, nesta precedência: `energy` < 39 → Quies; senão, se Somnium ativo e (`seism` ≥ 16 ∨ não noturno ∨ toque) → agnitio (REQ-L4-11); senão, se Vigilia ∧ noturno ∧ `seism` < 2 ∧ `idle_s` ≥ 1200 ∧ r ≥ r∞ ∧ (`semen` ∧ 0xFFF) = 0 → Somnium; senão, se AmorFati, mantém até o fecho; senão Vigilia. | `arbitrium` | U `modus_precedence` |
| REQ-L4-11 | Agnitio: `ascensus` = 0; `curvatura` = 0; `conatus` +32; `eventus` = 12; `ramus` = 0; `gradus` = 0; `modus` = Vigilia. Em Somnium, `curvatura` −3 por tick até −64; em AmorFati, +2 por tick até +64; em Vigilia, ×3/4 por tick em direção a 0. | `arbitrium` | U `agnitio_resets`; U `curvature_dynamics` |
| REQ-L4-12 | Aforismos DEVEM usar notação modal: □ (necessidade) e ◊ (contingência), com uma única linha por nó de ramificação e sem texto explicativo. | `arbitrium` | I `aphorism_format` |

## L5 — Signo e forma

| ID | Requisito | Módulo | Verificação |
| --- | --- | --- | --- |
| REQ-L5-01 | O pixel DEVE ser `u8`: bits 0..=1 classe de estrato; 2..=3 rótulo de aresta (0 convexa, 1 côncava, 2 oclusiva, 3 nenhuma); 4..=7 luminância 0..=15. | `signum` | U `pixel_fields` |
| REQ-L5-02 | A paleta DEVE ter 16 entradas RGB derivadas do estado: base ardósia (18, 20, 24) a giz (240, 240, 245); `tranquillitas` reduz o croma; `onus` > 200 desloca para sépia; Somnium desloca para azul frio; AmorFati coloca ouro (255, 214, 102) no tom 15; `festum` eleva o tom 15 em 8 %. | `signum` | U `palette_laws` |
| REQ-L5-03 | Stili: `Creta` desenha linhas por Bresenham e preenche por pontilhado, com o tom e a classe de cada elemento dados pela tabela de `05-estetica.md` §3; `Musivum` rasteriza em W/s × H/s e amplia por vizinho mais próximo; `Littera` rasteriza em (colunas, linhas) e mapeia luminância para a rampa de 10 glifos ` .:-=+*#%@`, preservando as linhas de texto: no raster, cada célula de 6×10·s acima do plinto, inclusive as parciais da borda direita e da faixa sobre o plinto, vira o glifo da média dos pixels existentes; `Bicolor` aplica a matriz de Bayer 4×4 empacotada `0x5D7F_91B3_6E4C_A280` (índice (y ∧ 3) ≪ 2 \| (x ∧ 3), 4 bits por limiar) sobre a luminância, com os extremos fixos: tom ≤ 1 → 0; tom ≥ 13 → 15; entre eles, 15 onde a matriz acende o tom e 0 nos demais. | `signum` | U `bayer_packed_matrix`; U `ramp_matches_ordo`; U `bicolor_only_two_tones`; G por stilus |
| REQ-L5-04 | A fonte DEVE conter maiúsculas A–Z, dígitos, pontuação `.,:;!?-+=/()[]'"*#%@·`, diacríticos ÁÀÂÃÉÊÍÓÔÕÚÇ, símbolos □ ◊ → ¬ ∧ ∨ ∅ Δ ∞ e letras φ δ κ, em células de 6×10 px (glifo 5×7, 2 linhas de diacrítico, 1 de espaço). Caracteres fora da tabela DEVEM mapear para `·`. | `signum` | U `font_coverage`; I `font_no_empty_glyph` |
| REQ-L5-05 | As curvas de chicote DEVEM ser, para cada lado e t em 0..=255 (0 no canto inferior, 255 na margem superior): desvio horizontal X(t) = Σₖ Aₖ·sen(2π·(k·t/256 + θₖ + λₖ)), k = 1..=3, com o seno de Bhaskara e as amplitudes A = (0,55; 0,30; 0,15) em Q16.16 (⌊Aₖ·2¹⁶⌋, decrescentes, soma ≤ 1), a fase θₖ = (`semen` girado à esquerda por 5k bits)/65 536 de volta e λₖ = k/4 de volta no lado direito, 0 no esquerdo; altura Y(t) = t/255, subida linear dos cantos inferiores à margem superior. X DEVE murchar ×(1024 − `onus`)/1024 e ×3/4 com energia < 77. Cada lado liga por retas os pontos de t = 0, 3, …, 255 em x = ⌊m′/2⌋ + ⌊X·(⌊m′/2⌋ − 1)⌋ à esquerda e W − ⌊m′/2⌋ − ⌊X·(⌊m′/2⌋ − 1)⌋ à direita, y = H − m′ − ⌊Y·(H − 2m′)⌋, com m′ = max(m, 2); a espessura DEVE ser 2·s quando `tranquillitas` < 32 e s caso contrário. | `signum` | U `whiplash_wilt`; U `whiplash_harmonics`; U `whiplash_amplitudes` |
| REQ-L5-06 | O grafo do céu DEVE ter: tronco em (m, H·φ⁻²/2); um nó por linha de crônica revelada em posição por dispersão de hash(`eventus`, `ramus`, i) na região do céu; aresta Bresenham ao nó anterior; operação por ramus: 0 linear, 1 bifurcação, 2 fractal (2 sub-ramos). Nós de constelação por dispersão de hash(`automatum`, j), j < 12. | `signum` | G `sky_graph_golden` |
| REQ-L5-07 | Hexagrama: 6 linhas de `tranquillitas` (bit 0 embaixo; 1 = contínua; 0 = quebrada) no canto superior direito do céu, visível com energia ≥ 77 e somente quando a região do céu tem largura e altura ≥ 12·s. | `signum` | U `hexagram_lines`; U `hexagram_needs_room` |
| REQ-L5-08 | Lua: disco na região do céu, visível quando noturno, menos um disco de sombra deslocado pela fração iluminada (à esquerda no crescente e à direita no minguante no hemisfério norte; espelhada com `australis`); a parte iluminada é pontilhada: o pixel recebe o tom da lua onde a matriz de Bayer o acende nesse tom e o tom 9 nos demais. | `signum` | U `moon_phase_fill`; U `moon_hemisphere_mirror`; G `golden_frames` (cenários noturnos) |
| REQ-L5-09 | Quadros-ouro: 4 cenários fixos (vigilia diurna; acedia ao crepúsculo; somnium noturno; quies) DEVEM reproduzir hashes registrados nos testes. | `mundus` | G `golden_frames` |
| REQ-L5-11 | Todo canal de forma (paleta, contraste mínimo, espessura de linha, cadência de marcha, registro do texto, curvatura aplicada, índice sonoro) DEVE ler apenas o `Concentus`; `concentus(&Mundus, &Facultates, &Indicia, &Regimen) -> Concentus` DEVE ser função pura; `palette(&Concentus)` DEVE depender só do concentus; dois stili na mesma situação DEVEM produzir o mesmo `Concentus`. | `concentus` | U `concentus_pure`; U `concentus_invariant_under_stilus` |
| REQ-L5-12 | Harmonia corrompida: com `onus` > 200, a borda direita da região do céu DEVE ser ⌊W·(φ⁻¹ ⊕ `0x1337`)⌋ em lugar de ⌊W·φ⁻¹⌋ (a fórmula de REQ-L2-11). | `signum` | U `corrupted_harmony_constant`; U `harmony_shapes_sky` |
| REQ-L5-10 | Espaço negativo: com `onus` > 200, os nós do grafo DEVEM ser posicionados sobre o contorno de uma das máscaras 16×16 (ampulheta, olho, balança), escolhida por `eventus` mod 3. | `signum`, `sensus` | U `negative_space_mask_snap` |

## L6 — Percepção

| ID | Requisito | Módulo | Verificação |
| --- | --- | --- | --- |
| REQ-L6-01 | Com `occluded`, o tick DEVE aplicar uma mutação ao `automatum`: ⊕ `0xAA` se `onus` > 128, senão rotação circular de 1 bit. O hospedeiro DEVE chamar `step` no máximo uma vez por período de oclusão. | `sensus` | U `occlusion_mutation` |
| REQ-L6-02 | Anomalia liminar: com `automatism` ≥ 200 ∧ `returned_ms` < 3000, a sombra da figura DEVE apontar para o lado oposto à luz. | `sensus`, `signum` | U `liminal_anomaly_window` |
| REQ-L6-03 | Evento raro: janela de 4 s ativa quando ctz(hash(`epoch_s` ≫ 2, `clausurae`, `eventus`)) ≥ 13; a cena mostra uma linha de tom 15 do ponto de Hénon no céu até a figura. | `sensus`, `signum` | U `rare_gate_probability` (frequência ≈ 2⁻¹³ em 2²⁰ janelas) |
| REQ-L6-04 | Inibição de retorno: o quadrante do nó do céu do evento k DEVE ser (3·k) mod 4, percorrendo os 4 quadrantes antes de repetir. | `sensus` | U `inhibition_of_return_cycle` |
| REQ-L6-05 | Nenhum mecanismo de L6 PODE depender de contagem de interações nem aumentar a frequência de qualquer evento com a interação. | `sensus` | I revisão da tabela de leis |

## L7 — Níveis de narração

| ID | Requisito | Módulo | Verificação |
| --- | --- | --- | --- |
| REQ-L7-01 | A cada toque (x, y) o quadrante q = ((y ≫ 7) ≪ 1) \| (x ≫ 7) DEVE atualizar a linha `quadrans` do `oraculum`: a célula q recebe min(peso + 1, 3); as demais células da linha recebem max(peso − 1, 0) quando a soma da linha excede 6; `quadrans` = q. | `transumptio` | U `oracle_update` |
| REQ-L7-02 | A armadilha DEVE estar pronta quando, na linha `quadrans`, o peso máximo = 3 e a soma ≤ 5; a cena mostra o ensō no centro do quadrante previsto. | `transumptio`, `signum` | U `trap_ready_condition` |
| REQ-L7-03 | Se a armadilha está pronta e o toque seguinte cai no quadrante previsto, `modus` = AmorFati, `eventus` = 11, `ramus` = 0, `gradus` = 0 e a linha do oráculo volta a `0x55`. | `transumptio` | U `metalepsis_fires` |
| REQ-L7-04 | A amostragem da rocha em contemplação DEVE devolver bit macro = bit(8v + u) do `Verbum` e bit micro = bit(8v' + u') de rotl(`Verbum`, 13) ⊕ `0x9E37_79B9_7F4A_7C15`, com (u', v') = ((u·z) ≫ 4, (v·z) ≫ 4) ∧ 7. | `transumptio` | U `abyme_sampling` |

## L8 — Memória

| ID | Requisito | Módulo | Verificação |
| --- | --- | --- | --- |
| REQ-L8-01 | As ruínas DEVEM ser popcount(bits 1..=13 de `clausurae`) colunas quebradas, posicionadas sobre a encosta por dispersão de hash(j). | `signum` | U `ruins_positions_deterministic` |
| REQ-L8-02 | Após agnitio, o canto superior direito DEVE exibir os 16 dígitos hexadecimais do `Verbum` por 240 ticks (enquanto `eventus` = 12 ∧ `gradus` < 240): no raster, a partir de (W − m − 96·s, m), em tom 10; nas células, como glifos na linha 0, alinhados à direita, em tom 10. | `signum` | U `cellae_agnitio_digits`; G `agnitio_golden` |

## Consilium

| ID | Requisito | Módulo | Verificação |
| --- | --- | --- | --- |
| REQ-C-01 | `Facultates` DEVE conter: `width`, `height`, `color_bits` ∈ {1, 4, 8, 24}, `max_hz`, `touch`, `motion`, `occlusion`, `persistence`, `cells`, `australis`, `tz_offset_min`, `lingua`, `compute` 0..=255. | `hospes` | U `facultates_defaults` |
| REQ-C-02 | `Regimen` DEVE conter: `stilus`, `strata` 1..=4, `tick_ms` (cadência do mundo: 42, 125, 250 ou 60 000), `interval_ms` (cadência de apresentação, ≥ `tick_ms`), `layers` (bits: céu, moldura, crônicas, personae, ruínas, impossibilia, hexagrama, pontilhado), `lines` 0..=6. | `consilium` | U `regimen_fields` |
| REQ-C-03 | `consilium(&Facultates, &Mundus, energy) -> Regimen` DEVE maximizar valor(regimen, situação) − λ·custo(regimen, facultates), λ = 1/8, sujeito a custo ≤ orçamento(energia, compute), por busca satisfatória: nível de aspiração = 80 % do valor máximo teórico, e, se inatingível dentro do orçamento, o melhor de 64 propostas; recozimento com temperatura T = energia/16; propostas geradas pelo gerador de Galois a partir de `semen`; determinístico. | `consilium` | U `consilium_deterministic`; U `consilium_respects_budget`; U `consilium_reaches_aspiration` |
| REQ-C-04 | Forçamentos, nesta precedência: hospedeiro só com células → Littera; `color_bits` = 1 → Bicolor; Quies → Bicolor e `tick_ms` = 60 000; Somnium → `strata` ≥ 2 e stilus ≠ Littera (salvo hospedeiro só com células); `max_hz` = 0 → `interval_ms` ≥ 60 000 com `tick_ms` pela faixa de energia; AmorFati → Creta preferido (bônus de valor, não forçamento). | `consilium` | U `consilium_forcings` |
| REQ-C-05 | O hospedeiro DEVE reavaliar `consilium` quando a faixa de energia (REQ-L1-07), as facultates ou o `modus` mudam; NÃO DEVE reavaliar a cada tick. | hospedeiros | I documentação dos hospedeiros |

## Limiar

| ID | Requisito | Módulo | Verificação |
| --- | --- | --- | --- |
| REQ-H-01 | `Indicia` DEVE conter: `epoch_ms` (i64), `energy`, `seism`, `automatism`, `idle_s` (u32), `returned_ms` (u32), `occluded`, `leaving`, `touch` (opcional, 0..=255 por eixo), `offset` (128 = centro), `tilt` (i8). | `hospes` | U `indicia_defaults` |
| REQ-H-02 | `Tabula` DEVE ser um raster de `width`·`height` pixels `u8` fornecido pelo hospedeiro; `Cellae` DEVE ser uma grade de células `u16` (bits 0..=7 glifo; 8..=11 tom; 12..=13 estrato), com o índice de glifo 0..=69 para a posição em `ORDO`, 70..=81 para ÁÀÂÃÉÊÍÓÔÕÚÇ, nesta ordem, e 57 (`·`) para os demais caracteres. O núcleo NÃO DEVE escrever fora do prefixo de `width`·`height` (ou `cols`·`rows`); `render_cellae` DEVE recusar, sem escrever, um raster auxiliar de outras dimensões. | `hospes`, `signum` | P `render_never_exceeds_slice`; P `pictor_writes_only_the_prefix`; U `warp_keeps_inside_frame` |
| REQ-H-03 | O ABI C DEVE expor: `sisyphium_version`, `sisyphium_genesis`, `sisyphium_parse`, `sisyphium_format`, `sisyphium_consilium`, `sisyphium_step`, `sisyphium_advance`, `sisyphium_render`, `sisyphium_palette`, com estruturas `#[repr(C)]` documentadas em `04-arquitetura.md` §6. | `limen` | U `ffi_roundtrip` |
| REQ-H-04 | `render` DEVE ser determinístico: mesmo mundo, facultates, regimen e indicia produzem o mesmo raster em qualquer plataforma. | `mundus` | G (hashes iguais em CI macOS e Linux) |
| REQ-H-05 | Tempo por quadro na máquina de referência (computador de mesa com processador ARM64 de 2020 ou posterior, 320×180): Creta ≤ 2 ms; Musivum ≤ 1 ms; Littera ≤ 1 ms; Bicolor ≤ 2 ms. 1920×1080 com s = 6: ≤ 30 ms. | `tabula` | M no PR do checkpoint |
| REQ-H-06 | `Mundus::advance(&mut self, &Indicia, ticks, tick_ms, &Regimen)` DEVE aplicar `ticks` chamadas de `step` com `epoch_ms` interpolado por `tick_ms` e os demais indicia constantes, e DEVE devolver a união dos `Nuntius`. Hospedeiros estáticos DEVEM chamar `advance` com ticks = min(decorrido_ms / `tick_ms`, 65 536). | `mundus`, hospedeiros | U `advance_equals_repeated_step`; I |

## Direitos e privacidade

| ID | Requisito | Módulo | Verificação |
| --- | --- | --- | --- |
| REQ-D-01 | Nenhum hospedeiro DEVE abrir conexões de rede. | hospedeiros | I varredura de símbolos e dependências |
| REQ-D-02 | O único dado persistido DEVE ser o texto de 32 dígitos hexadecimais. Nenhum registro de interação é gravado. | hospedeiros | I |
| REQ-D-03 | Nenhum nome de produto, marca, motor, loja, rede ou serviço nos textos da obra, no código do núcleo e nos ativos. Nos hospedeiros e na documentação, apenas nomes de sistemas operacionais e de utilitários do sistema, como instrução técnica. | todos | I varredura por lista de termos vetados no CI |
