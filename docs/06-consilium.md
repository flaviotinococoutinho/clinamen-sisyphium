# 06 — Consilium

## 1. Função

`consilium(&Facultates, &Mundus, energy) -> Regimen` é a camada entre a situação narrativa e a
forma visível. Recebe `Facultates` (medidas do meio: dimensões, cor, cadência, entradas,
computação), a energia e o `Mundus` (situação: modus, onus, tranquillitas, curvatura, ramus,
eventus, clausurae) e devolve um `Regimen`. O hospedeiro reavalia quando mudam as facultates, a
faixa de energia, o modus, o orçamento calculado ou a chave semântica definida em REQ-C-05.
O orçamento pode mudar dentro da mesma faixa de energia. A cada medida e após a evolução do
mundo, o hospedeiro compara essas condições; a busca ocorre nas transições, sem execução
incondicional em todo tick.

## 2. Espaço de regimina

| Dimensão | Valores |
| --- | --- |
| stilus | Creta, Musivum, Littera, Bicolor |
| strata | 1..=4 |
| tick_ms | 42, 125, 250, 60 000 (fixado pela faixa de energia; REQ-L1-07) |
| interval_ms | 42, 125, 250, 1000, 60 000, com interval_ms ≥ tick_ms |
| layers | subconjunto de {céu, moldura, crônicas, personae, ruínas, impossibilia, hexagrama, pontilhado} |
| lines | 0..=6 |

Cardinalidade: 4 × 4 × 5 × 256 × 7 = 143 360 regimina.

## 3. Custo

Custo em unidades de trabalho por segundo. A tabela define as unidades de um quadro de
referência de 320×180. O custo por quadro soma a base, os estratos adicionais, as camadas
habilitadas e o acréscimo do stilus.

| Componente | Custo por quadro a 320×180 |
| --- | --- |
| base (ardósia, encosta 1 estrato, figura, rocha) | 10 |
| estrato adicional | 3 |
| céu | 6 |
| moldura | 5 |
| crônicas (por linha) | 1 |
| personae | 2 |
| ruínas | 1 |
| impossibilia | 4 |
| hexagrama | 1 |
| pontilhado | 4 |
| Littera (mapa) | 2 |
| Bicolor (limiar) | 3 |

`cost = ⌊unidades · área · 1000 / (57 600 · interval_ms)⌋`, com os produtos em `u64` e resultado
saturado em `u32::MAX`. Um intervalo de 0 ms custa `u32::MAX`. A área é W·H em Creta, Littera e
Bicolor. Em Musivum, a área é ⌊W/s⌋·⌊H/s⌋, com s = max(1, min(⌊W/320⌋, ⌊H/180⌋)) da composição
(REQ-L2-11): o quadro é desenhado na escala lógica antes da ampliação. Crônicas só custam quando
ANNALES está habilitada, uma unidade por linha.

Orçamento: `budget = compute · energy / 64` unidades por segundo. Exemplo: computador de mesa
(compute 255) ligado à energia (255): 1016 unidades/s. Telefone (compute 96) a 30 % (77): 115
unidades/s. As divisões inteiras descartam a fração; um quadro por minuto pode custar zero
unidades/s.

## 4. Valor semiótico

Valor por camada, modulado pela situação.

| Componente | Valor base | Modulação |
| --- | --- | --- |
| céu | 8 | ×2 durante clinamen ativo (`ramus` ≠ 3) |
| moldura | 5 | ×2 com `onus` > 128 (murcha visível) |
| crônicas | 2 por linha | ×2 durante clinamen ativo |
| personae | 6 | ×0 fora de `eventus` 13..=15 |
| ruínas | 3 | ×0 quando popcount(`clausurae` ≫ 1) = 0 |
| impossibilia | 7 | ×0 fora das condições (`conatus` > 200 ∧ `onus` > 200, ou Somnium) |
| hexagrama | 2 | ×2 com `tranquillitas` ≥ 48 |
| pontilhado | 2 | ×2 em Somnium |
| estratos | 2 por estrato além do primeiro | ×2 em Somnium |
| cadência | 42 ms: 6; 125 ms: 4; 250 ms: 2; 1000 ms: 1; 60 000 ms: 0 | ×0 para 42 ms com energia < 179 |
| stilus | Creta 4; Musivum 3; Littera 2; Bicolor 1 | Creta 8 em AmorFati; Littera 6 na redução eidética; Bicolor 6 em Quies |

A redução eidética ocorre com energia 39..=76 e `tranquillitas` ≥ 48. AmorFati soma 4 ao valor
de Creta; constitui preferência, sem fixar o stilus. Energia < 39 força a forma Bicolor
(REQ-L1-07), mas o valor 6 pertence ao modus Quies: a energia não muda o modo lido pelo cálculo
de valor.

`value_max` soma os máximos por dimensão na situação presente, sob os forçamentos: quatro
estratos; todas as camadas de valor positivo com seis linhas; a cadência permitida de maior
valor; o stilus permitido de maior valor. O orçamento não entra nesse máximo. O nível de
aspiração é 80 % desse valor (Simon, 1955), comparado por `5 · value ≥ 4 · value_max`, sem
arredondamento. Se o orçamento impedir esse nível, a busca devolve o maior escore encontrado em
até 64 propostas.

## 5. Busca

Os forçamentos seguem esta precedência (REQ-C-04): `cells` fixa Littera; senão `color_bits` = 1
fixa Bicolor; senão Quies ou energia < 39 fixa Bicolor; senão Somnium substitui Littera por
Creta. Somnium exige ao menos dois estratos em qualquer meio, inclusive em células. Quies ou
energia < 39 fixa `tick_ms` em 60 000; nos demais casos vale `band_tick_ms(energy)`.

O piso de apresentação é o menor elemento de `INTERVALS = [42, 125, 250, 1000, 60_000]` maior ou
igual a max(`tick_ms`, ⌈1000/`max_hz`⌉). Com `max_hz` = 0, o piso é 60 000 ms. Os forçamentos
elevam cada intervalo proposto a esse piso.

O regimen piso (`floor`) tem o stilus forçado, ou Musivum quando livre; um estrato, dois em
Somnium; nenhuma camada; nenhuma linha; `interval_ms` = 60 000; `tick_ms` pela faixa, ou
60 000 em Quies. A busca devolve esse piso imediatamente quando o orçamento calculado é zero ou
quando o custo do piso excede o orçamento. O retorno com orçamento zero vale mesmo quando o
truncamento do custo permitiria camadas adicionais de custo zero.

A base tem Creta, quatro estratos, seis linhas, somente as camadas de valor positivo na
situação, `tick_ms` pela faixa e o piso de apresentação. Os forçamentos se aplicam à base. O
estado inicial é a base quando seu custo cabe no orçamento; caso contrário é o regimen piso.

O escore é `8 · value − cost`, calculado em `i64`, equivalente a valor − λ·custo com λ = 1/8. O
melhor regimen começa no estado inicial. A temperatura T = ⌊energia/16⌋ permanece constante.
Cada uma das até 64 iterações segue esta ordem (Kirkpatrick et al., 1983):

1. A busca para se o valor do melhor regimen atinge o nível de aspiração.
2. O gerador g, iniciado em `semen`, avança por Galois; g mod 5 escolhe a dimensão. O gerador
   avança de novo; `pick` = g ≫ 4 escolhe o valor da proposta.
3. A proposta troca uma dimensão: stilus `[Creta, Musivum, Littera, Bicolor][pick mod 4]`;
   estratos `1 + pick mod 4`; intervalo `INTERVALS[pick mod 5]`; camadas com o bit `pick mod 8`
   invertido; linhas `pick mod 7`. Os forçamentos se aplicam depois da troca.
4. A proposta igual ao regimen corrente ou acima do orçamento é descartada.
5. Δ = escore corrente − escore proposto. A proposta é aceita com Δ ≤ 0. Com Δ > 0 e T = 0, é
   rejeitada. Com Δ > 0 e T > 0, g avança e a proposta é aceita se `(g ∧ 0xFF) < EXP[k]`, com
   k = min(15, ⌊4·Δ/T⌋) (Metropolis et al., 1953).
6. A proposta aceita vira corrente. O melhor regimen muda somente se o escore proposto é
   estritamente maior que o melhor escore. Ao encerrar, a busca devolve o melhor regimen.

`EXP[k] = ⌊255·e^(−k/4)⌋`, para k = 0..=15:

| k | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| EXP[k] | 255 | 198 | 154 | 120 | 93 | 73 | 56 | 44 | 34 | 26 | 20 | 16 | 12 | 9 | 7 | 5 |

A tabela ocupa `u128` = `0x0507_090C_1014_1A22_2C38_495D_789A_C6FF`; o byte k é
`(EXP >> (8 · k)) & 0xFF`. A aceitação usa somente inteiros. A mesma combinação de facultates,
mundo e energia produz o mesmo regimen; o gerador da busca não altera o mundo.

## 6. Adaptação ao sistema

O hospedeiro traduz versões de sistema e APIs em `Facultates`: por exemplo, canvas acelerado
disponível → `compute` maior; ausência de leitura de bateria → `energy` = 255 e `max_hz` conforme
a política de economia do sistema; ausência de eventos de visibilidade → `occlusion` = false;
painel de 1 bit → `color_bits` = 1; terminal → `cells` = true. O núcleo nunca recebe nomes de
sistemas, versões ou APIs.
