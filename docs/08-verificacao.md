# 08 — Verificação

## 1. Classes de verificação

| Classe | Forma | Onde |
| --- | --- | --- |
| Lei (U) | teste unitário nomeado pela lei; compara com valores derivados da fonte | `crates/sisyphium/src/<modulo>.rs`, `#[cfg(test)]` |
| Propriedade (P) | casos gerados pelo gerador de Galois do próprio núcleo (65 535 sementes) combinados com vetores fixos de `Indicia`; invariantes de domínio e totalidade | `crates/sisyphium/tests/` |
| Quadro-ouro (G) | hash FNV-1a de 64 bits do raster para cenários fixos; igual em todas as plataformas do CI | `crates/sisyphium/tests/aurea.rs` |
| Inspeção (I) | lints de restrição; varredura de termos vetados; cobertura da fonte; formato dos aforismos | `cargo clippy`; `scripts/verba.sh` |
| Medição (M) | tempo por quadro, tamanho de binário, tamanho de estado | `sisyphium-tabula --metire`; registrado no PR |

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
cargo build --release -p sisyphium-tabula -p sisyphium-textus
cargo build --release -p sisyphium-limen --target wasm32-unknown-unknown   # quando o alvo existe
scripts/verba.sh                                                           # termos vetados
scripts/metire.sh                                                          # medições para o PR
```

## 4. Integração contínua

Dois sistemas (Linux, macOS). Etapas: formato; lints; testes; documentação; construção dos
hospedeiros; varredura de termos vetados; comparação dos hashes dos quadros-ouro entre os dois
sistemas. Uma etapa falha interrompe o merge.

## 5. Varredura de termos vetados

`scripts/verba.sh` procura em `crates/`, `hosts/`, `docs/` e `README.md` os termos de uma
lista mantida em `scripts/verba-vetita.txt`: nomes de produtos comerciais, marcas, motores,
lojas, redes, serviços, ferramentas de produção e doutrinas de engenharia. Ficam fora da lista os
nomes de sistemas operacionais, de linguagens e de utilitários invocados pelos scripts de
hospedeiros. Ficam fora da varredura a própria lista, `docs/referencias.md` (bibliografia) e a
configuração da integração contínua. A lista é o único lugar do repositório, além desses, onde os
termos aparecem.

## 6. Medições

Máquina de referência: computador de mesa com processador ARM64 de 2020 ou posterior; modelo e
versão do sistema registrados no PR. Protocolo: 1000 quadros por stilus a 320×180 e 100 quadros a 1920×1080, mediana e p95 em
microssegundos; tamanho do binário `sisyphium-tabula` em bytes após `strip`; tamanho de
`Mundus` em bytes (16); número de linhas do corpus e comprimento máximo.

## 7. Cenários dos quadros-ouro

| Cenário | `Mundus` | `Indicia` | `Regimen` |
| --- | --- | --- | --- |
| vigilia diurna | genesis(1) após 4 096 ticks | 2026-06-21 12:00 local; energia 255; sismo 0 | Creta, 4 estratos, 42 ms, todas as camadas, 6 linhas |
| acedia ao crepúsculo | genesis(2) após 4 096 ticks com onus forçado a 160 | 2026-09-22 18:30; energia 120 | Musivum, 2 estratos, 125 ms |
| somnium noturno | genesis(3) com modus Somnium e curvatura −48 | 2026-12-21 02:30; energia 200; ociosidade 3 600 s | Creta, 4 estratos |
| quies | genesis(4) | 2026-03-20 09:00; energia 20 | Bicolor, 1 estrato, 60 000 ms |

Os hashes são constantes nos testes; uma mudança de forma intencional atualiza as constantes no
mesmo commit que a causa.
