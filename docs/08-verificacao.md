# 08 — Verificação

## 1. Classes de verificação

| Classe | Forma | Onde |
| --- | --- | --- |
| Lei (U) | teste unitário nomeado pela lei; compara com valores derivados da fonte | `#[cfg(test)]` do módulo; integração pública em `crates/sisyphium/tests/chronologia.rs` (ordem das leis) e `crates/sisyphium/tests/quies.rs` (visibilidade da encosta, REQ-L5-11) |
| Propriedade (P) | casos gerados pelo gerador de Galois do próprio núcleo (65 535 sementes) combinados com vetores fixos de `Indicia`; invariantes de domínio e totalidade | `#[cfg(test)]` do módulo |
| Quadro-ouro (G) | hash FNV-1a de 64 bits do raster para cenários fixos; igual em todas as plataformas do CI | `crates/sisyphium/tests/aurea.rs` |
| Inspeção (I) | lints de restrição; varredura de termos vetados; cobertura da fonte; formato dos aforismos | `cargo clippy`; `scripts/verba.sh` |
| Medição (M) | tempo por quadro, tempo de avanço, tamanho da biblioteca e tamanho de estado | `metire_frames` em `crates/sisyphium/tests/aurea.rs`; construção da biblioteca em release; registrado no PR |

## 2. Lints de restrição do núcleo

`clippy::float_arithmetic`, `clippy::float_cmp`, `clippy::indexing_slicing`,
`clippy::arithmetic_side_effects`, `clippy::unwrap_used`, `clippy::expect_used`,
`clippy::panic`, `clippy::unreachable`, `clippy::todo`, `clippy::unimplemented`,
`clippy::missing_docs_in_private_items`, `unsafe_code` (forbid), `missing_docs` (deny), e
`clippy::disallowed_methods` com `i8::abs`, `i16::abs`, `i32::abs` e `i64::abs` vetados em
`clippy.toml`. Os testes ficam fora das restrições de pânico e da proibição de `abs`.

## 3. Comandos

```
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo doc --workspace --no-deps
scripts/verba.sh
cargo test --release -p sisyphium --test aurea -- --ignored metire_frames --nocapture --test-threads=1
cargo build --release -p sisyphium --lib
```

## 4. Integração contínua

Dois sistemas (Linux, macOS). Etapas: formato; lints; testes; documentação e varredura de
termos vetados. Os testes comparam os quadros-ouro com as mesmas constantes nos dois sistemas.
Uma execução adicional verifica os testes na versão mínima 1.85. Uma etapa falha interrompe o
merge.

## 5. Varredura de termos vetados

`scripts/verba.sh` procura em `crates/`, `hosts/`, `docs/` e `README.md` os termos de uma
lista mantida em `scripts/verba-vetita.txt`: nomes de produtos comerciais, marcas, motores,
lojas, redes, serviços, ferramentas de produção e doutrinas de engenharia. Ficam fora da lista os
nomes de sistemas operacionais, de linguagens e de utilitários invocados pelos scripts de
hospedeiros. Ficam fora da varredura a própria lista, `docs/referencias.md` (bibliografia) e a
configuração da integração contínua. A lista é o único lugar do repositório, além desses, onde os
termos aparecem.

## 6. Medições

Máquina de referência: computador de mesa com processador ARM64 de 2020 ou posterior; modelo,
versão do sistema e compilador registrados no PR. A medição DEVE usar release e ocorrer sem
compilações concorrentes.

`metire_frames` mede cinco famílias de §7: vigilia, acedia, somnium, quies e littera, cada uma
com 1000 quadros a 320×180 e 100 quadros a 1920×1080. Cada intervalo inclui somente
`Mundus::render`; construção do mundo, alocação, paleta, conversão de imagem, hash e disco ficam
fora do intervalo. Uma renderização de aquecimento precede cada conjunto. Argumentos e resultado
atravessam `black_box`. As amostras em microssegundos são ordenadas; mediana no índice n/2 e
p95 no índice ⌊95·n/100⌋, com índices iniciados em zero. Os resultados DEVEM ser registrados
por família e dimensão, sem combinar amostras de mundos distintos. Os limites são os de
REQ-H-05.

A mesma execução mede `advance` com 65 536 ticks de 42 ms, a partir de genesis(1), com época
final 1 782 054 000 000 e demais `Indicia` padrão: mediana de 11 execuções em microssegundos,
índice 5 após ordenação. O tamanho de `Mundus` DEVE ser 16 bytes. A rotina unitária
`metire_advance` permite medição isolada da mesma operação.

O PR registra também o tamanho em bytes e a soma SHA-256 da biblioteca `.rlib` produzida por
`cargo build --release -p sisyphium --lib`, além do número de linhas do corpus e do comprimento
máximo em glifos. A biblioteca inclui código e metadados; seu tamanho NÃO DEVE ser apresentado
como tamanho de executável. A medição de executáveis dos hospedeiros pertence ao checkpoint 4.

## 7. Cenários dos quadros-ouro

Os quadros têm 320×180 pixels, salvo indicação, e as horas são locais (−03:00). Todos usam
`Facultates::default()`, salvo dimensões. O `Regimen` padrão é Creta, 4 estratos, `tick_ms` e
`interval_ms` de 42 ms, todas as camadas e 6 linhas. Os campos não indicados de `Indicia` e
`Regimen` mantêm os valores padrão. Cada quadro usa `Mundus::render`, com as crônicas escolhidas
pelo estado; a preparação usa apenas a API pública.

| Cenário | `Mundus` | `Indicia` | `Regimen` |
| --- | --- | --- | --- |
| vigilia diurna | genesis(1) após 4 096 ticks de 42 ms até a época indicada | 2026-06-21 12:00 local; energia 255; sismo 0 | Creta, 4 estratos, 42 ms, todas as camadas, 6 linhas |
| acedia ao crepúsculo | genesis(2) após 4 096 ticks de 125 ms até a época indicada; depois onus 160 | 2026-09-22 18:30; energia 120 | Musivum, 2 estratos, 125 ms |
| somnium noturno | genesis(3) com modus Somnium e curvatura −48 | 2026-12-21 02:30; energia 200; ociosidade 3 600 s | Creta, 4 estratos |
| quies | genesis(4) com modus Quies | 2026-03-20 09:00; energia 20 | Bicolor, 1 estrato, 60 000 ms |
| littera | o mundo de vigilia diurna | como vigilia diurna | Littera, demais campos como vigilia diurna |
| somnium 1920×1080 | o mundo de somnium noturno | como somnium noturno | Creta, 4 estratos, quadro de 1920×1080 |
| grafo linear | genesis(1), eventus 3, ramus 0, gradus 400 | 2026-06-21 12:00 | padrão |
| grafo bifurcado | como grafo linear, ramus 1 | como grafo linear | padrão |
| grafo fractal | como grafo linear, ramus 2 | como grafo linear | padrão |
| grafo com máscara | como grafo fractal, onus 230 | como grafo linear | padrão |
| noite | genesis(5), ascensus 60 | 2026-12-24 02:30; energia 200 | padrão |
| agnitio | genesis(6), eventus 12, ramus 0, gradus 100 | 2026-06-21 12:00 | padrão |

Os testes `golden_frames`, `sky_graph_golden`, `night_golden` e `agnitio_golden` cobrem,
respectivamente, os primeiros seis quadros, os quatro grafos, a noite e a agnitio. O hash usa
os bytes do raster codificado, antes da conversão pela paleta. A rotina ignorada
`dump_golden_ppm` exporta os 12 quadros em `CARGO_TARGET_TMPDIR/aurea-cp3`; os PPMs DEVEM ser
convertidos em PNG e inspecionados antes de registrar hashes.

Os hashes são constantes nos testes; uma mudança de forma intencional atualiza as constantes no
mesmo commit que a causa.
