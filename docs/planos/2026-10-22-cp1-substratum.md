# Plano — Checkpoint 1: substrato e natureza

**Meta:** o crate `sisyphium` existe, compila em `no_std`, e contém os níveis L0, L1 e L2 com
`mundus` parcial (genesis, format, parse), o contrato `hospes`, lints de restrição, integração
contínua e varredura de termos vetados.

**Especificação:** `docs/03-especificacao.md` (REQ-L0-01..06, REQ-L1-01..08, REQ-L2-01..11,
REQ-C-01, REQ-C-02, REQ-H-01, REQ-H-02, REQ-D-01..03); `docs/04-arquitetura.md` §1–§4.

**Pilha:** Rust, edição 2024, versão mínima 1.85; sem dependências em todo o workspace.

## Restrições globais

- `crates/sisyphium`: `#![no_std]`, sem `alloc`, `#![forbid(unsafe_code)]`,
  `#![deny(missing_docs)]`, `[lints.clippy]` com `float_arithmetic`, `float_cmp`,
  `indexing_slicing`, `arithmetic_side_effects`, `unwrap_used`, `expect_used`, `panic`,
  `unreachable`, `todo`, `unimplemented`, `missing_docs_in_private_items` em `deny`; `pedantic`
  em `warn`. Módulos de teste anulam só as restrições de pânico e de aritmética.
- Identificadores de função em inglês; tipos, módulos e campos de estado em latim conforme o
  glossário. Documentação de API em inglês com a fonte da lei.
- Aritmética explícita: `wrapping_*`, `saturating_*`, `checked_*`. Acesso a fatias por `get`.
- Commits `tipo(escopo): assunto` em inglês. Sem trailers.

## Foco de revisão

1. `Verbum::new(0)` e qualquer palavra com `semen` = 0: normalização para `0xACE1` em toda
   entrada (`new`, `parse`, `genesis`). Teste em `verbum::tests::zero_semen_normalized`.
2. `parse` com 32 dígitos maiúsculos, com 31 ou 33 caracteres, com `g`: aceita o primeiro,
   rejeita os demais. Teste `mundus::tests::parse_rejects_invalid`.
3. `Ephemeris` em `epoch_ms` negativo e em fusos negativos: fase e dia do ano corretos por
   divisão euclidiana. Teste `ephemeris::tests::negative_epoch_and_offset`.
4. `Fx::mul` com operandos extremos (`i32::MIN`, `i32::MAX`): saturação sem estouro. Teste
   `spatium::tests::fixed_mul_div_saturate`.
5. `Compositio::new` com largura abaixo de 272 px ou `lines` = 0: regiões não negativas e sem
   sobreposição. Teste `spatium::tests::composition_small_widths`.

---

### Tarefa 1: workspace, lints, integração contínua e varredura

**Arquivos:**
- Criar: `Cargo.toml` (workspace: `members = ["crates/*"]`, `resolver = "3"`), `rustfmt.toml`
  (`edition = "2024"`, `max_width = 100`), `clippy.toml` (vazio com comentário de propósito),
  `crates/sisyphium/Cargo.toml`, `crates/sisyphium/src/lib.rs`,
  `.github/workflows/ci.yml`, `scripts/verba.sh`, `scripts/verba-vetita.txt`,
  `scripts/verify.sh`.

**Interfaces:**
- Produz: crate `sisyphium` com módulos públicos declarados em `lib.rs` nas tarefas seguintes.

- [ ] Escrever `lib.rs` com os atributos de crate e a documentação de crate (três frases: o que é
      o núcleo, a finitude do estado, a ausência de E/S).
- [ ] `cargo build -p sisyphium` → compila sem avisos.
- [ ] Escrever `ci.yml`: matriz `ubuntu-latest`, `macos-latest`; etapas `cargo fmt --all
      --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test
      --workspace`, `cargo doc --workspace --no-deps`, `scripts/verba.sh`.
- [ ] Escrever `scripts/verba.sh`: `grep -rniF -f scripts/verba-vetita.txt crates hosts docs
      README.md --exclude=referencias.md`; sai com 1 se houver ocorrência.
- [ ] Escrever `scripts/verba-vetita.txt` com os termos vetados (uma por linha).
- [ ] Executar `scripts/verba.sh` → sem ocorrências; `scripts/verify.sh` encadeia os comandos de
      `08-verificacao.md` §3.
- [ ] Commit: `build: workspace, lints and continuous integration`.

### Tarefa 2: `verbum`

**Arquivos:** criar `crates/sisyphium/src/verbum.rs`; registrar `pub mod verbum;` em `lib.rs`.

**Interfaces:**
- Produz: `#[repr(transparent)] pub struct Verbum(u64)`; `pub enum Modus { Vigilia = 0, Quies
  = 1, Somnium = 2, AmorFati = 3 }`; `impl Verbum { pub const fn new(bits: u64) -> Self; pub
  const fn bits(self) -> u64; pub const fn semen(self) -> u16; conatus/onus/ascensus → u8;
  curvatura → i8; automatum → u8; tranquillitas → u8 (0..=63); modus → Modus; with_semen(self,
  u16) -> Self (0 → 0xACE1); with_conatus… with_modus }`; constantes `SEMEN_DEFAULT: u16 =
  0xACE1`.

- [ ] Teste `verbum_pack_unpack_identity`: para 65 535 sementes geradas por `natura::galois`
      (tarefa 4; até lá, por iteração aritmética `s = s.wrapping_mul(0x9E37).wrapping_add(1)`),
      construir `Verbum::new(x)` com `x` variado em todos os campos e conferir que cada acessor
      devolve o campo embutido e que `with_*` seguido do acessor é identidade.
- [ ] `cargo test -p sisyphium verbum` → falha por símbolo ausente.
- [ ] Implementar deslocamentos e máscaras como `const` nomeadas por campo (`SEMEN_SHIFT`,
      `SEMEN_MASK`, …) e acessores `const fn`.
- [ ] Teste `zero_semen_normalized`: `Verbum::new(0).semen() == 0xACE1`;
      `Verbum::new(0).with_semen(0).semen() == 0xACE1`.
- [ ] `cargo test -p sisyphium verbum` → passa.
- [ ] Commit: `feat(verbum): 64-bit present-state word with field laws`.

### Tarefa 3: `memoria`

**Arquivos:** criar `crates/sisyphium/src/memoria.rs`.

**Interfaces:**
- Produz: `#[repr(transparent)] pub struct Memoria(u64)`; acessores `clausurae() -> u16`
  (14 bits), `quadrans() -> u8`, `eventus() -> u8`, `ramus() -> u8`, `gradus() -> u16`
  (10 bits), `oraculum() -> u32`; `with_*`; `pub const RAMUS_EXSPECTANS: u8 = 3`.

- [ ] Teste `memoria_pack_unpack_identity` (mesmo esquema da tarefa 2, todos os 64 bits).
- [ ] Implementar; `cargo test -p sisyphium memoria` → passa.
- [ ] Commit: `feat(memoria): 64-bit past-and-intent word with field laws`.

### Tarefa 4: `natura`

**Arquivos:** criar `crates/sisyphium/src/natura.rs`.

**Interfaces:**
- Consome: `spatium::Fx` (tarefa 6); ordem de implementação: tarefa 6 antes desta.
- Produz: `pub const GALOIS_POLY: u16 = 0xB400`; `pub const fn galois(semen: u16) -> u16`;
  `pub const fn rule30(cells: u8) -> u8`; `pub const fn wind(cells: u8) -> bool`;
  `pub const fn pink(t32: u64) -> u8`; `pub const R_INF: Fx` (`0x391E8`);
  `pub const fn logistic(x: Fx, r: Fx) -> Fx`; `pub const fn henon(x: Fx, y: Fx) -> (Fx, Fx)`;
  `pub const fn mix(x: u64) -> u64`.

- [ ] Teste `galois_full_period`: a partir de 1, contar iterações até voltar a 1 → 65 535; a
      partir de `0xACE1`, idem.
- [ ] Teste `rule30_known_sequence`: de `0x10`, as 4 gerações seguintes conferem com a regra
      aplicada manualmente (`0x38`, `0x64`, `0xF6`, `0x1B`, valores recalculados no teste por uma
      implementação de referência bit a bit).
- [ ] Teste `pink_octave_rates`: a oitava k de `pink` (função interna `octave(k, t)`) muda de
      valor exatamente quando `t >> k` muda. Teste `pink_mean_bounds`: média de 4 096 amostras em
      96..=160.
- [ ] Teste `logistic_fixed_point`: `logistic(Fx::from_ratio(1,2), Fx::from_ratio(4,1)) ==
      Fx::from_int(1)`; `henon_bounded`: 4 096 iterações de (0,0) ficam em |x| ≤ 1,5, |y| ≤ 0,5.
- [ ] Implementar; `cargo test -p sisyphium natura` → passa.
- [ ] Commit: `feat(natura): galois generator, rule 30, pink noise, chaotic maps`.

### Tarefa 5: `ephemeris`

**Arquivos:** criar `crates/sisyphium/src/ephemeris.rs`.

**Interfaces:**
- Produz: `pub struct Ephemeris { pub phase: u8, pub season: u8, pub luna: u8, pub nocturnal:
  bool, pub festum: bool, pub day_of_year: u16 }`; `pub const fn ephemeris(epoch_ms: i64,
  tz_offset_min: i16, australis: bool) -> Ephemeris`; `pub const SYNODIC_S: i64 = 2_551_443`;
  `pub const LUNA_EPOCH_S: i64 = 947_182_440`.

- [ ] Teste `ephemeris_known_dates`: 2026-06-21 12:00 em fuso −180 → `day_of_year` 172,
      `festum` true, `phase` 128 ± 1, `nocturnal` false, `season` ≈ 65 no norte e ≈ 193 no sul;
      2000-01-06 18:14 UTC → `luna` 0; 2026-12-21 02:30 → `nocturnal` true, `festum` true.
- [ ] Teste `negative_epoch_and_offset`: 1969-12-31 23:00 UTC em fuso +60 → dia 1 de 1970,
      fase 0.
- [ ] Implementar com divisão euclidiana (`rem_euclid`, `div_euclid`) e anos bissextos pela
      regra gregoriana.
- [ ] Commit: `feat(ephemeris): circadian, seasonal and lunar phases from epoch`.

### Tarefa 6: `spatium`

**Arquivos:** criar `crates/sisyphium/src/spatium.rs`.

**Interfaces:**
- Produz: `#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)] pub struct Fx(pub
  i32)`; `impl Fx { pub const ONE; pub const fn from_int(i32); pub const fn from_ratio(num:
  i32, den: i32); pub const fn to_int(self) -> i32; pub const fn mul(self, Fx) -> Fx; pub const
  fn div(self, Fx) -> Fx; pub const fn add/sub saturating }`; constantes `PHI`, `PHI_INV`,
  `PHI_INV2`, `PHI_INV3`, `PHI_INV4`, `SILVER`, `PI`, `KNUTH32: u32`, `KNUTH64: u64`;
  `pub const fn sin(angle: u16) -> Fx`, `cos`; `pub const fn isqrt(n: u32) -> u32`;
  `pub const fn cordic(x: Fx, y: Fx, angle: u16) -> (Fx, Fx)`; `pub const fn scatter32(k: u32,
  bits: u32) -> u32`; `pub const fn scatter64(k: u64, bits: u32) -> u64`; `pub const fn warp(x:
  i16, y: i16, kappa: i8) -> (i16, i16)`; `pub const STRATA: [Fx; 4]`; `pub const DOMINANCE_LUT:
  u16 = 0x0A9C`; `pub const fn dominates(new: u8, old: u8) -> bool`; `pub const fn dimetric(x:
  i32, y: i32, z: i32) -> (i32, i32)`; `pub const fn penrose(step: u32, cx: i32, cy: i32,
  span: u32) -> (i32, i32, u8)`; `pub struct Compositio { pub s, pub m, pub plinth, pub
  scene_h, pub sky: Rect, pub slope: (Pt, Pt), pub annales: Rect, pub glyphs_per_line, pub
  figure_h, pub boulder_d }`; `pub const fn compositio(w: u16, h: u16, lines: u8) ->
  Compositio`.

- [ ] Testes, um por lei, com os nomes de `03-especificacao.md`: `fixed_mul_div_saturate`,
      `bhaskara_max_error` (compara com `f64::sin` no teste; erro ≤ 110), `isqrt_floor`
      (propriedade sobre 65 535 valores dispersos e os extremos), `cordic_unit_rotation`,
      `knuth_constants`, `curvature_identity_at_zero`, `curvature_sign_direction`,
      `strata_velocities_phi` (compara com `f64` no teste), `stratum_dominance_cycle`,
      `stratum_lut_matches_rule`, `isometric_kernel`, `penrose_closed_loop`,
      `composition_constants` (320×180, lines 6 → s 1, m 16, plinth 76, scene_h 104, sky y
      ≤ 39, slope de (122, 104) a (304, 39), 48 glifos), `composition_small_widths`.
- [ ] Implementar; `cargo test -p sisyphium spatium` → passa.
- [ ] Commit: `feat(spatium): fixed point, projections, strata and composition`.

### Tarefa 7: `hospes`

**Arquivos:** criar `crates/sisyphium/src/hospes.rs`.

**Interfaces:**
- Produz: `Lingua { En, Pt }`, `Stilus { Creta, Musivum, Littera, Bicolor }`, `Facultates`,
  `Indicia`, `Regimen`, `Tabula<'a>`, `Cellae<'a>`, `Nuntius(pub u8)` com constantes de bits,
  constantes de camadas (`CAELUM`, `MARGO`, `ANNALES`, `PERSONAE`, `RUINAE`, `IMPOSSIBILIA`,
  `HEXAGRAMMA`, `PUNCTA`); `impl Default` para `Facultates` (320×180, 8 bits, 24 Hz, sem
  entradas, `australis` true, fuso −180, `En`, compute 255) e `Indicia` (época 0, energia 255,
  demais zero, `idle_s` `u32::MAX`, `returned_ms` `u32::MAX`).

- [ ] Testes `facultates_defaults`, `indicia_defaults`, `regimen_fields` (campos e domínios).
- [ ] Implementar; commit: `feat(hospes): host contract types`.

### Tarefa 8: `mundus` (genesis, format, parse)

**Arquivos:** criar `crates/sisyphium/src/mundus.rs`.

**Interfaces:**
- Produz: `pub struct Mundus { pub verbum: Verbum, pub memoria: Memoria }`; `pub const fn
  genesis(seed: u64) -> Mundus`; `pub const fn format(&self) -> [u8; 32]`; `pub const fn
  parse(text: &[u8]) -> Option<Mundus>`.

- [ ] Testes `genesis_fields` (valores de REQ-L0-04), `format_parse_roundtrip` (65 535 mundos
      gerados por `galois` e `mix`), `parse_rejects_invalid` (31, 33 caracteres; `g`; aceita
      maiúsculas).
- [ ] Implementar; `cargo test -p sisyphium` → tudo passa.
- [ ] Commit: `feat(mundus): genesis and 32-digit persistence`.

### Tarefa 9: fechamento do checkpoint

- [ ] `scripts/verify.sh` → todas as etapas limpas; registrar contagens (testes, linhas,
      tamanho do crate) para o PR.
- [ ] Atualizar `README.md`: "Checkpoint presente: 1".
- [ ] Remover este plano; commit: `docs: close checkpoint 1`.
- [ ] PR `feat(core): substratum and nature (checkpoint 1)` com o corpo de `convencoes.md` §5;
      merge por rebase; tag `cp/1-substratum`.
