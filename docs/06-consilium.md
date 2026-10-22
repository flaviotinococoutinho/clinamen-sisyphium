# 06 — Consilium

## 1. Função

`consilium(&Facultates, &Mundus, energy) -> Regimen` é a camada entre a situação narrativa e a
forma visível. Recebe `Facultates` (medidas do meio: dimensões, cor, cadência, entradas,
computação), a energia e o `Mundus` (situação: modus, onus, tranquillitas, curvatura, ramus,
eventus, clausurae) e devolve um `Regimen`. O hospedeiro reavalia
quando a faixa de energia, as facultates ou o modus mudam.

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

Custo em unidades de trabalho por segundo, derivado de medições do checkpoint 4 e documentado na
tabela abaixo. O custo total é a soma dos custos das camadas vezes o fator de área (W·H/57 600)
vezes a cadência (1000/interval_ms).

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

Orçamento: `budget = compute · energy / 64` unidades por segundo. Exemplo: computador de mesa
(compute 255) ligado à energia (255): 1016 unidades/s. Telefone (compute 96) a 30 % (77): 115
unidades/s.

## 4. Valor semiótico

Valor por camada, modulado pela situação.

| Componente | Valor base | Modulação |
| --- | --- | --- |
| céu | 8 | ×2 durante clinamen ativo (`ramus` ≠ 3) |
| moldura | 5 | ×2 com `onus` > 128 (murcha visível) |
| crônicas | 2 por linha | ×2 durante clinamen ativo |
| personae | 6 | ×0 fora de persona |
| ruínas | 3 | ×0 sem ruínas |
| impossibilia | 7 | ×0 fora das condições (`conatus` > 200 ∧ `onus` > 200, ou Somnium) |
| hexagrama | 2 | ×2 com `tranquillitas` ≥ 48 |
| pontilhado | 2 | ×2 em Somnium |
| estratos | 2 por estrato além do primeiro | ×2 em Somnium |
| cadência | 42 ms: 6; 125 ms: 4; 250 ms: 2; 1000 ms: 1; 60 000 ms: 0 | ×0 para 42 ms com energia < 179 |
| stilus | Creta 4; Musivum 3; Littera 2; Bicolor 1 | Littera 6 na redução eidética; Bicolor 6 em Quies |

Valor máximo teórico: soma dos máximos. Nível de aspiração: 80 % do valor máximo teórico
(Simon, 1955); se inatingível dentro do orçamento, a busca devolve o melhor das 64 propostas.

## 5. Busca

1. Forçamentos (REQ-C-04) fixam dimensões obrigatórias.
2. Estado inicial: regimen base da faixa de energia (REQ-L1-07) com todas as camadas.
3. 64 iterações de recozimento (Kirkpatrick et al., 1983): proposta = vizinho que altera uma
   dimensão escolhida pelo gerador de Galois a partir de `semen`; aceita se valor − λ·custo
   melhora ou se `(semen ∧ 0xFF) < 255·exp(−Δ/T)` aproximado por tabela inteira de 16 entradas,
   com T = energia/16; rejeita propostas acima do orçamento.
4. Para quando o valor atinge o nível de aspiração ou ao fim das 64 iterações; devolve o melhor
   encontrado. λ = 1/8.
5. Determinismo: mesmas entradas, mesmo regimen.

## 6. Adaptação ao sistema

O hospedeiro traduz versões de sistema e APIs em `Facultates`: por exemplo, canvas acelerado
disponível → `compute` maior; ausência de leitura de bateria → `energy` = 255 e `max_hz` conforme
a política de economia do sistema; ausência de eventos de visibilidade → `occlusion` = false;
painel de 1 bit → `color_bits` = 1; terminal → `cells` = true. O núcleo nunca recebe nomes de
sistemas, versões ou APIs.
