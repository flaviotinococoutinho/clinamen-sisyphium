# Plano — Checkpoint 3: escolha, percepção e narração

**Meta:** o mundo passa a viver. `Mundus::step` e `Mundus::advance` aplicam, numa ordem fixa,
as leis de `arbitrium` (clinamina, crônicas em inglês e português, personae, modus, agnitio,
fecho, interrupção), `sensus` (oclusão, anomalia liminar, porta rara, inibição de retorno) e
`transumptio` (oráculo, armadilha, metalepse, abismo); `consilium` escolhe o `Regimen`;
`Mundus::render` desenha o texto do estado, o grafo do céu com o aforismo, as máscaras de espaço
negativo, as personae, o ensō, o raio raro, a sombra liminar, as figuras impossíveis, o kintsugi
e o abismo da rocha; os quadros-ouro passam aos cenários de `08-verificacao.md` §7.

**Arquitetura:** leis puras (`const fn` sempre que possível) sobre `Verbum` e `Memoria` em
`arbitrium`, `sensus` e `transumptio`; `mundus` compõe as leis num tick e não contém lei
própria; `signum` lê o estado só para geometria e lê o `Concentus` para forma; `consilium` é uma
busca determinística sobre 143 360 regimina.

**Tecnologia:** Rust 2024, MSRV 1.85, crate `sisyphium` sem dependências.

**Especificação:** `docs/03-especificacao.md` (REQ-L0-05, REQ-L1-04, REQ-L1-07, REQ-L1-08,
REQ-L4-01..12, REQ-L5-06, REQ-L5-08, REQ-L5-09, REQ-L5-10, REQ-L5-12, REQ-L6-01..05,
REQ-L7-01..04, REQ-L8-02, REQ-C-01..04, REQ-H-06); `docs/04-arquitetura.md` §2–§7;
`docs/06-consilium.md`; `docs/07-narrativa.md`; `docs/08-verificacao.md` §1 e §7.

## Decisões do plano

Cada decisão resolve uma lacuna ou contradição da especificação; a tarefa que a implementa
reescreve o documento afetado para que ele descreva o estado presente.

| # | Decisão | Razão | Custo se errada |
| --- | --- | --- | --- |
| D1 | `Mundus::step(&mut self, &Facultates, &Indicia) -> Nuntius` e `Mundus::advance(&mut self, &Facultates, &Indicia, ticks: u32, tick_ms: u32) -> Nuntius`; sem `&Regimen` | fase circadiana e noturno (REQ-L1-04, L4-09, L4-10) exigem `tz_offset_min` e `australis`; nenhuma lei de estado lê a forma | assinatura do ABI no checkpoint 4 |
| D2 | Ordem do tick: gerador; fecho; interrupção; oclusão; lasca; toque; modus; passo; leis contínuas; evento; curvatura (limitada a ±64); porta rara | determinismo exige uma ordem; o fecho antes do passo deixa o cume visível por um tick; a curvatura depois do evento mantém o domínio operacional mesmo com a visita de Heráclito (+8) | reordenação e novos quadros-ouro |
| D3 | Somnium sem estímulo persiste (cláusula ausente em REQ-L4-10) | sem ela o sonho termina no tick seguinte sem agnitio | uma linha da lei do modus |
| D4 | O daemon usa a porta das demais personae: `ramus` = 3 ∧ (`semen` ∧ 0x1FF) = 0 | sem porta, a persona se repete a cada fim de evento e soma `onus` +16 a cada 240 ticks | frequência do daemon |
| D5 | Eventos especiais (`eventus` 8..=15) terminam em `gradus` ≥ 240: `eventus` = (`semen` ≫ 8) ∧ 7, `ramus` = 3, `gradus` = 0 | a especificação não diz qual clinamen sucede uma persona, agnitio ou metalepse | sucessão de eventos |
| D6 | "retirar (−30 durante 3 min)" vira Δonus −30 único | o estado não guarda duração | uma entrada da tabela |
| D7 | Contemplação = Quies ∨ persona (`eventus` 13..=15) ∨ sono (`eventus` = 12 ∧ `gradus` < 64); em contemplação não há passo | REQ-L4-06 nomeia a contemplação; a figura contemplativa não sobe | ritmo da subida |
| D8 | Toque em Somnium desperta (agnitio) e atualiza o oráculo, sem metalepse | o despertar precede a profecia | um ramo da lei do toque |
| D9 | No mesmo tick, clinamen precede persona | o clinamen é o fio principal; personae ocupam os intervalos | frequência de personae |
| D10 | "`onus` −1 em direção a 32" move um passo em direção a 32 (sobe quando abaixo) | leitura literal de "em direção a" | deriva do peso |
| D11 | Corpus: cinco linhas por ramus, uma por degrau (linha i no degrau i + 1); metalepse com 1 linha (abstrato); agnitio com 2 (íntimo, abstrato); interrupção com 4 (epocal, epocal, civil, civil); "sem pronomes" = sem pronomes pessoais, inclusive oblíquos átonos, e sem possessivos; texto no Apêndice A | estrutura uniforme permite o registro pela posição e torna a escada verificável | reescrita do corpus |
| D12 | consilium: valor máximo teórico = soma dos máximos por dimensão na situação presente e sob os forçamentos; estado inicial só com camadas de valor positivo; regimen piso; Musivum com área (W/s)·(H/s); tabela EXP do Apêndice B; AmorFati soma 4 ao valor de Creta | sem isso o nível de aspiração é inatingível na maioria das situações e o estado inicial excede o orçamento | tabelas recalibradas no checkpoint 4 |
| D13 | `Registrum` passa a `arbitrium` (reexportado por `concentus`); `concentus.registrum` = registro da última linha visível | o registro é propriedade do texto | caminho do tipo |
| D14 | Quadros-ouro em `crates/sisyphium/tests/aurea.rs` pela API pública, cenários de §7 mais grafo, noite e agnitio | `08-verificacao.md` §1 já fixa o lugar | hashes |
| D15 | Máscaras de espaço negativo (Apêndice C) num quadrado de lado min(qw, qh) centrado no quadrante do evento | satisfaz REQ-L5-10 e REQ-L6-04 ao mesmo tempo | posição do desenho |
| D16 | As duas linhas de `semen` na textura da rocha usam tons 7 (aceso) e 5 (apagado) | `semen` muda a cada tick; com contraste pleno a rocha cintila a 24 Hz; abaixo do limiar diferencial (Fechner, 1860) o grão vive sem chamar a atenção (harness moral) | brilho de duas linhas da rocha |
| D17 | `contrastus` governa o fundo: tom máximo da encosta e das ruínas = 15 − `contrastus`; `contrastus` = 15 em Quies | o canal existia sem leitor; "menor energia, maior contraste" (`05-estetica.md` §10) vira um mundo mais escuro em volta da figura | tons do fundo |
| D18 | `sonus` = ⌊255·8/√(64 + `onus`)⌋ (massa = `onus` + 64, a rocha) | lei "altura ∝ 1/√massa" de `05-estetica.md` §10 | um índice sem síntese na 0.1 |
| D19 | Sob expansão (κ > 0), o pixel lógico (x, y) ocupa o retângulo das projeções vizinhas, de P(x, y) a (Pₓ(x + 1, y) − 1, P_y(x, y + 1) − 1), com lado mínimo 1; substitui o bloco 1 + ⌈3κr²⌉ do checkpoint 2 | o bloco vale 2 para qualquer κ > 0 fora do centro: o traço dobra a partir de κ = 1 e afina de súbito quando a curvatura volta a 0; a cobertura pelas vizinhas é exata e contínua em κ | espessura sob expansão |
| D20 | O plinto das crônicas fica fora da projeção de curvatura; o resto da cena passa por ela | a narração é extradiegética (Genette, 1972): o sonho curva o mundo, não a voz que o narra; sob κ = −48 a contração fundia as primeiras palavras das linhas; o texto reto sob um mundo curvo revela o sonho sem anunciá-lo (`07-narrativa.md` §6) | o sonho deixa de tocar o texto |

## Restrições globais

- Crate `sisyphium` (núcleo): `#![no_std]`, sem `alloc`, sem dependências, sem `unsafe`, sem ponto
  flutuante (`clippy::float_arithmetic` proibido), sem pânico alcançável (aritmética saturante ou
  modular explícita; acesso a fatias verificado), `#![deny(missing_docs)]`.
- Lints do crate (`lib.rs`): `indexing_slicing`, `arithmetic_side_effects` e
  `missing_docs_in_private_items` negados; liberados só na compilação de testes, salvo
  `missing_docs_in_private_items`: toda função de teste e todo auxiliar levam `///`.
- Cada função implementa uma lei nomeada e documentada com a fonte. Toda constante tem derivação
  registrada e teste que a confere. Toda lei tem ao menos um teste nomeado pela lei.
- Tipos e campos em latim; funções, variáveis, rustdoc e mensagens de commit em inglês; documentos
  em português; nenhum nome de produto, marca, motor ou doutrina de engenharia.
- Sem alocação: todo buffer vem do hospedeiro; o núcleo lê e escreve só o prefixo de
  `width`·`height` (e `cols`·`rows`) (REQ-H-02).
- `abs` de inteiros com sinal é proibido (`clippy.toml`, `disallowed-methods`): usar
  `unsigned_abs` ou `saturating_abs`.
- `scripts/verify.sh` limpo ao fim de cada tarefa (formato, clippy pedantic com `-D warnings`,
  testes, rustdoc com `-D warnings`, varredura de termos vetados).
- Commits `tipo(escopo): assunto` em inglês, minúsculas, no imperativo, sem trailers.
- Uma mudança de forma intencional atualiza os hashes-ouro no mesmo commit, após inspeção visual
  dos quadros (PPM convertido em PNG).

## Foco de revisão

1. `advance` com `ticks` = 65 536, `tick_ms` = 60 000 e `epoch_ms` perto de `i64::MIN` e de
   `i64::MAX`: nenhum pânico; épocas saturadas (`advance_saturates_epoch`, Tarefa 4).
2. Estados arbitrários de 32 dígitos (`eventus` 8..=10, `ramus` 3 com `eventus` ≥ 8,
   `curvatura` −128, `gradus` 1023): `step` total, eventos especiais terminam, nenhum índice fora
   das tabelas (`step_is_total`, Tarefa 4; `annales_arbitrary_memoria`, Tarefa 1).
3. Toque nas bordas dos quadrantes (127/128 em cada eixo) e toque com `Facultates.touch` = false
   (`oracle_update`, Tarefa 3: o toque é medida e vale sempre).
4. `consilium` com orçamento 0 (energia 0 ou `compute` 0) e 1920×1080: devolve o regimen piso;
   `interval_ms` nunca abaixo de 1000/`max_hz` (`consilium_respects_budget`, Tarefa 5).
5. Céu degenerado (272×60 com 6 linhas; altura do céu ≤ 1 px): grafo, máscaras e aforismo não
   dividem por zero nem escrevem fora do quadro (`degenerate_sky`, Tarefa 6).

---

### Tarefa 1: `arbitrium` — tabela de eventos e corpus

**Arquivos:** criar `crates/sisyphium/src/arbitrium/mod.rs`, `arbitrium/annales_en.rs`,
`arbitrium/annales_pt.rs`; modificar `crates/sisyphium/src/lib.rs` (`pub mod arbitrium;`),
`crates/sisyphium/src/concentus.rs` (remove a definição de `Registrum`, acrescenta
`pub use crate::arbitrium::Registrum;`), `docs/07-narrativa.md` (§3 regra de pronomes; §4
Δonus de "retirar" e lista de civilizações; §8 forma das tabelas).

**Interfaces:**
- Consome: `Memoria` (`eventus`, `ramus`, `gradus`, `clausurae`), `Lingua`, `natura::mix`.
- Produz:
  - `pub enum Registrum { Intimum, Vivum, Civile, Epochale, Abstractum }` (movido, mesmos derives).
  - `pub const OFFSETS: [u16; 6] = [24, 72, 144, 240, 360, 480];`
  - `pub const METALEPSIS: u8 = 11; AGNITIO: u8 = 12; DIOGENES: u8 = 13; HERACLITUS: u8 = 14; DAEMON: u8 = 15;`
  - `pub const fn band(eventus: u8) -> (u8, u8)` — faixa de `ascensus`; `(0, 0)` fora de 0..=7.
  - `pub const fn delta(eventus: u8, ramus: u8) -> (i8, i8, i8)` — (Δonus, Δconatus, Δtranquillitas); `(0, 0, 0)` fora da tabela.
  - `pub const fn revealed(gradus: u16) -> u8` — linhas de crônica visíveis, 0..=5.
  - `pub const fn registrum_of(line: u8) -> Registrum` — 0 Intimum … 4 e acima Abstractum.
  - `pub struct Annales { pub lines: [&'static str; 6], pub count: u8, pub registrum: Registrum, pub aphorismus: Option<&'static str> }` (`Clone, Copy, Debug, PartialEq, Eq`).
  - `pub fn annales(memoria: Memoria, lingua: Lingua, leaving: bool) -> Annales`.
  - Módulos `annales_en` e `annales_pt` com `ANNALES: [[[&str; 5]; 3]; 8]`, `APHORISMI: [&str; 8]`, `PERSONAE: [&str; 3]`, `AGNITIO: [&str; 2]`, `METALEPSIS: &str`, `INTERRUPTIO: [&str; 4]`.

Faixas (`band`), em ordem de `eventus`: (8, 40), (90, 115), (48, 72), (120, 150), (160, 190),
(20, 50), (200, 230), (60, 100); empacotadas numa constante `u128` de 16 bits por evento
(`lo | hi ≪ 8`). Deltas por `eventus` e `ramus`:

| `eventus` | ramus 0 | ramus 1 | ramus 2 |
| --- | --- | --- | --- |
| 0 | (12, 0, −2) | (6, −4, 0) | (0, −2, 2) |
| 1 | (5, −8, 4) | (−30, 0, 0) | (45, −10, −6) |
| 2 | (0, 2, 2) | (4, 0, 0) | (8, −6, −4) |
| 3 | (0, −1, 0) | (2, 0, 1) | (−4, 4, −2) |
| 4 | (0, −3, 1) | (3, −1, 0) | (20, −8, −3) |
| 5 | (−6, −4, 6) | (0, −2, 1) | (16, 0, −2) |
| 6 | (2, 6, −1) | (0, 0, 3) | (−8, 2, 4) |
| 7 | (1, −6, 0) | (0, 3, 2) | (6, −2, −4) |

Seleção de `annales` (lei única, nesta ordem):
- `eventus` 0..=7 ∧ `ramus` 0..=2: linhas `ANNALES[eventus][ramus][i]` para i < `revealed(gradus)`;
  registro = `registrum_of(count − 1)`, ou `Intimum` sem linha; `aphorismus` =
  `APHORISMI[eventus]` quando `gradus` ≥ 24.
- `eventus` 11: `METALEPSIS` com `gradus` ≥ 24; registro `Abstractum`.
- `eventus` 12: `AGNITIO[0]` com `gradus` ≥ 24 (`Intimum`); `AGNITIO[1]` com `gradus` ≥ 72
  (`Abstractum`).
- `eventus` 13..=15: `PERSONAE[eventus − 13]` com `gradus` ≥ 24; registros `Vivum`,
  `Abstractum`, `Epochale`.
- `ramus` 3 com `eventus` 0..=7, e `eventus` 8..=10: nenhuma linha; `Intimum`.
- `leaving`: acrescenta `INTERRUPTIO[k]`, k = `mix(clausurae | eventus ≪ 16) & 3`, como última
  linha (substitui a sexta quando há seis); registro `Epochale` para k ∈ {0, 1}, `Civile` para
  k ∈ {2, 3}.

- [ ] **Passo 1: testes que falham** em `arbitrium/mod.rs`:

```rust
/// Eight clinamina with their bands and branch deltas; reserved events carry their lines
/// (REQ-L4-01).
#[test]
fn event_table_complete() {
    let bands = [(8, 40), (90, 115), (48, 72), (120, 150), (160, 190), (20, 50), (200, 230), (60, 100)];
    for (e, b) in (0u8..).zip(bands) {
        assert_eq!(band(e), b);
    }
    assert_eq!(band(8), (0, 0));
    assert_eq!(delta(1, 1), (-30, 0, 0));
    assert_eq!(delta(1, 2), (45, -10, -6));
    assert_eq!(delta(7, 2), (6, -2, -4));
    assert_eq!(delta(9, 0), (0, 0, 0));
    for table in [annales_en::ANNALES, annales_pt::ANNALES] {
        assert!(table.iter().flatten().flatten().all(|l| !l.is_empty()));
    }
}

/// Line i of a chronicle becomes visible at offset i; five lines per branch (REQ-L4-04).
#[test]
fn annales_reveal_offsets() {
    assert_eq!(OFFSETS, [24, 72, 144, 240, 360, 480]);
    let cases = [(0, 0), (23, 0), (24, 1), (71, 1), (72, 2), (143, 2), (144, 3), (239, 3), (240, 4), (359, 4), (360, 5), (599, 5), (1023, 5)];
    for (g, n) in cases {
        assert_eq!(revealed(g), n, "gradus {g}");
    }
    let m = Memoria::new(0).with_eventus(2).with_ramus(1).with_gradus(150);
    let a = annales(m, Lingua::En, false);
    assert_eq!(a.count, 3);
    assert_eq!(a.lines[..3], annales_en::ANNALES[2][1][..3]);
    assert_eq!(a.registrum, Registrum::Civile);
    assert_eq!(a.aphorismus, Some(annales_en::APHORISMI[2]));
    assert_eq!(annales(m.with_gradus(23), Lingua::Pt, false).aphorismus, None);
}

/// Every line fits 40 glyphs of the font, in uppercase (`docs/07-narrativa.md` §8).
#[test]
fn annales_lines_within_width() { /* every string of both modules: chars().count() ≤ 40;
    every char is in `signum::littera::ORDO` or in "ÁÀÂÃÉÊÍÓÔÕÚÇ"; no char is lowercase */ }

/// No personal or possessive pronoun, clitics included (`docs/07-narrativa.md` §3).
#[test]
fn annales_impersonal() { /* split every string on spaces and hyphens, strip . , ; : ! ? " ( );
    EN words ∉ {I ME MY MINE MYSELF YOU YOUR YOURS HE HIM HIS SHE HER HERS IT ITS WE US OUR
    OURS THEY THEM THEIR THEIRS WHO WHOM WHOSE SOMEONE ANYONE EVERYONE NOBODY};
    PT words ∉ {EU MIM COMIGO TU TI CONTIGO VOCÊ VOCÊS ELE ELA ELES ELAS NÓS VÓS ME TE SE LHE
    LHES VOS MEU MINHA MEUS MINHAS TEU TUA SEU SUA SEUS SUAS NOSSO NOSSA DELE DELA QUEM
    NINGUÉM ALGUÉM} */ }

/// Aphorisms read `□X → ◊Y`, one word each side, at most 16 glyphs (REQ-L4-12).
#[test]
fn aphorism_format() { /* both APHORISMI: split on " → " gives exactly two parts; the first
    starts with '□', the second with '◊'; no space inside either part; chars().count() ≤ 16 */ }

/// Reserved events, the interruption and arbitrary memories select lines by the law above.
#[test]
fn annales_arbitrary_memoria() {
    let agn = Memoria::new(0).with_eventus(AGNITIO).with_ramus(0);
    assert_eq!(annales(agn.with_gradus(30), Lingua::En, false).registrum, Registrum::Intimum);
    let late = annales(agn.with_gradus(80), Lingua::En, false);
    assert_eq!((late.count, late.registrum), (2, Registrum::Abstractum));
    let her = annales(Memoria::new(0).with_eventus(HERACLITUS).with_gradus(30), Lingua::Pt, false);
    assert_eq!((her.lines[0], her.registrum), (annales_pt::PERSONAE[1], Registrum::Abstractum));
    let leave = annales(Memoria::new(0).with_eventus(4).with_ramus(3), Lingua::En, true);
    assert_eq!(leave.count, 1);
    assert!(annales_en::INTERRUPTIO.contains(&leave.lines[0]));
    for bits in [0u64, u64::MAX, 0x0000_0000_FFFF_FFFF, 0x8000_0000_0029_0000] {
        for leaving in [false, true] {
            let a = annales(Memoria::new(bits), Lingua::Pt, leaving);
            assert!(a.count <= 6);
        }
    }
}
```

- [ ] **Passo 2:** `cargo test -p sisyphium arbitrium` — falha por símbolos ausentes.
- [ ] **Passo 3:** implementar; os módulos `annales_en`/`annales_pt` transcrevem o Apêndice A
  sem alteração de uma letra; `concentus` reexporta `Registrum`.
- [ ] **Passo 4:** `cargo test -p sisyphium` — tudo verde, inclusive os testes de `concentus`.
- [ ] **Passo 5:** reescrever `docs/07-narrativa.md`: §3 "sem pronomes pessoais (inclusive
  oblíquos átonos) nem possessivos"; §4 "retirar (−30, 0, 0)" e lista das 24 civilizações do
  Apêndice A com épocas; §8 forma das tabelas e registro pela posição da linha.
- [ ] **Passo 6:** `scripts/verify.sh`; commit `feat(arbitrium): add the event table and the chronicles in two languages`.

### Tarefa 2: `arbitrium` — leis de escolha e consequência

**Arquivos:** modificar `crates/sisyphium/src/arbitrium/mod.rs`, `crates/sisyphium/src/natura.rs`
(`logistic_r`), `docs/03-especificacao.md` (REQ-L1-04 constante 0,6 = `0x999A`; REQ-L4-03
acrescenta D5; REQ-L4-05 acrescenta "em contemplação não há passo"; REQ-L4-09 acrescenta a
porta do daemon; REQ-L4-10 acrescenta a persistência do Somnium).

**Interfaces:**
- Consome: Tarefa 1 (`band`, `delta`, constantes de `eventus`); `natura::{galois, wind, R_INF}`;
  `Nuntius`.
- Produz (todas `pub const fn` em `arbitrium`):
  - `pace_threshold(v: Verbum) -> u16`; `paces(v: Verbum) -> bool` (REQ-L4-05, L1-08).
  - `continuous(v: Verbum, stepped: bool, contemplating: bool, seism: u8, idle_s: u32) -> Verbum` (REQ-L4-06, D10).
  - `clinamen_fires(v: Verbum, m: Memoria) -> bool`; `fire(v: Verbum, m: Memoria) -> (Verbum, Memoria)` (REQ-L4-02).
  - `age(m: Memoria) -> Memoria` (`gradus` +1 saturante em 1023); `close(v: Verbum, m: Memoria) -> Memoria` (REQ-L4-03); `release(v: Verbum, m: Memoria) -> Memoria` (D5).
  - `persona(v: Verbum, m: Memoria, phase: u8, idle_s: u32, occluded: bool) -> Option<u8>`; `visit(v: Verbum, m: Memoria, persona: u8) -> (Verbum, Memoria)` (REQ-L4-09, D4).
  - `event(v: Verbum, m: Memoria, phase: u8, idle_s: u32, occluded: bool) -> (Verbum, Memoria, Nuntius)`: `age`; se `eventus` < 8 ∧ `ramus` ≠ 3 ∧ `gradus` ≥ 600 → `close`; senão se `eventus` ≥ 8 ∧ `gradus` ≥ 240 → `release`; depois, se `clinamen_fires` → `fire` e `CLINAMEN`; senão, se `persona` → `visit` e `PERSONA` (D9).
  - `closure(v: Verbum, m: Memoria) -> (Verbum, Memoria)` (REQ-L4-07); `interrupt(m: Memoria) -> Memoria` (REQ-L4-08).
  - `somnium_eligible(nocturnal: bool, idle_s: u32) -> bool` (REQ-L1-04).
  - `agnitio(v: Verbum, m: Memoria) -> (Verbum, Memoria)` (REQ-L4-11).
  - `modus_law(v: Verbum, m: Memoria, energy: u8, seism: u8, nocturnal: bool, touched: bool, idle_s: u32) -> (Verbum, Memoria, Nuntius)` (REQ-L4-10, D3; `AGNITIO` quando desperta).
  - `curvature(v: Verbum) -> Verbum` (REQ-L4-11: Somnium −3 até −64; AmorFati +2 até +64; Vigilia e Quies ×3/4 truncado em direção a 0; saturante).
  - `asleep(m: Memoria) -> bool`; `contemplating(v: Verbum, m: Memoria) -> bool` (D7).
  - Em `natura`: `pub const fn logistic_r(nocturnal: bool, idle_s: u32) -> Fx` =
    `0x28000 + 0x999A·noturno + 0x999A·min(idle_s, 3600)/3600`.

Domínios: `conatus`, `onus` e `ascensus` saturam em 0..=255; `tranquillitas` em 0..=63;
`curvatura` satura em `i8`. Δ aplicados por soma saturante. T = clamp(`conatus` − ⌊`onus`/2⌋,
16, 255), ÷2 com vento, ≫ ⌊(`ascensus` − 240)/4⌋ com `ascensus` ≥ 240; `paces` exige
(`semen` ∧ 0x7FFF) < T, `ascensus` < 255 e, de 254 para 255, `conatus` ≥ 160 ∧ sem vento.

- [ ] **Passo 1: testes que falham:**

```rust
/// T = clamp(conatus − onus/2, 16, 255) (REQ-L4-05).
#[test]
fn pace_threshold() {
    let v = Verbum::new(0).with_conatus(192).with_onus(32).with_automatum(0x10);
    assert_eq!(super::pace_threshold(v), 176);
    assert_eq!(super::pace_threshold(v.with_conatus(10)), 16);
    assert_eq!(super::pace_threshold(v.with_conatus(255).with_onus(0)), 255);
    assert_eq!(super::pace_threshold(v.with_conatus(0).with_onus(255)), 16);
}

/// Six living cells make wind and halve the threshold (REQ-L1-08).
#[test]
fn wind_halves_pace() {
    let v = Verbum::new(0).with_conatus(192).with_onus(32);
    assert_eq!(pace_threshold(v.with_automatum(0b0001_1111)), 176);
    assert_eq!(pace_threshold(v.with_automatum(0b0011_1111)), 88);
}

/// Near the summit the threshold halves every four steps; the last step needs effort and calm
/// air (REQ-L4-05).
#[test]
fn zeigarnik_asymptote() {
    let top = Verbum::new(0).with_conatus(200).with_onus(0).with_automatum(0x10);
    for (a, t) in [(239, 200), (240, 200), (244, 100), (248, 50), (252, 25), (255, 25)] {
        assert_eq!(pace_threshold(top.with_ascensus(a)), t, "ascensus {a}");
    }
    let ready = top.with_ascensus(254).with_semen(1);
    assert!(paces(ready));
    assert!(!paces(ready.with_conatus(159)));
    assert!(!paces(ready.with_automatum(0xFF)));
    assert!(!paces(top.with_ascensus(255).with_semen(1)));
    assert!(!paces(top.with_ascensus(10).with_semen(200)));
}

/// The continuous laws of one tick, gated by the generator's low bits (REQ-L4-06).
#[test]
fn continuous_laws() {
    let b = Verbum::new(0).with_conatus(100).with_onus(100).with_tranquillitas(30);
    assert_eq!(continuous(b.with_semen(0x0004), true, false, 0, 0).conatus(), 99);
    assert_eq!(continuous(b.with_semen(0x0005), true, false, 0, 0).conatus(), 100);
    assert_eq!(continuous(b.with_semen(0x0200), false, false, 0, 0).conatus(), 101);
    assert_eq!(continuous(b.with_semen(0x0020), false, false, 0, 0).conatus(), 100);
    assert_eq!(continuous(b.with_semen(0x0020), false, true, 0, 0).conatus(), 101);
    assert_eq!(continuous(b.with_semen(0x0004), true, false, 128, 0).conatus(), 98);
    assert_eq!(continuous(b.with_semen(0x0040), false, false, 0, 0).onus(), 99);
    assert_eq!(continuous(b.with_onus(10).with_semen(0x0040), false, false, 0, 0).onus(), 11);
    assert_eq!(continuous(b.with_onus(32).with_semen(0x0040), false, false, 0, 0).onus(), 32);
    assert_eq!(continuous(b.with_semen(0x0100), false, false, 0, 600).tranquillitas(), 31);
    assert_eq!(continuous(b.with_semen(0x0100), false, false, 0, 599).tranquillitas(), 30);
    assert_eq!(continuous(b.with_semen(0x0001), false, false, 64, 0).tranquillitas(), 29);
    assert_eq!(continuous(b.with_conatus(0).with_semen(0x0004), true, false, 255, 0).conatus(), 0);
    assert_eq!(continuous(b.with_conatus(255).with_semen(0x0200), false, false, 0, 0).conatus(), 255);
    assert_eq!(continuous(b.with_tranquillitas(63).with_semen(0x0100), false, false, 0, 600).tranquillitas(), 63);
    assert_eq!(continuous(b.with_tranquillitas(0).with_semen(0x0001), false, false, 255, 0).tranquillitas(), 0);
}

/// A clinamen fires only while awaiting, inside its band, through the 24/256 gate (REQ-L4-02).
#[test]
fn clinamen_fires_in_band() {
    let m = Memoria::new(0).with_eventus(0).with_ramus(3);
    let v = Verbum::new(0).with_ascensus(8).with_semen(0x0017);
    assert!(clinamen_fires(v, m));
    assert!(!clinamen_fires(v.with_semen(0x0018), m));
    assert!(!clinamen_fires(v.with_ascensus(7), m));
    assert!(clinamen_fires(v.with_ascensus(40), m));
    assert!(!clinamen_fires(v.with_ascensus(41), m));
    assert!(!clinamen_fires(v, m.with_ramus(0)));
    assert!(!clinamen_fires(v, m.with_eventus(AGNITIO)));
    for e in 0..8u8 {
        let (lo, hi) = band(e);
        let me = m.with_eventus(e);
        assert!(clinamen_fires(v.with_ascensus(lo), me) && clinamen_fires(v.with_ascensus(hi), me));
        assert!(!clinamen_fires(v.with_ascensus(lo - 1), me) && !clinamen_fires(v.with_ascensus(hi + 1), me));
    }
}

/// Firing picks the branch by semen ≫ 12 mod 3, resets the age and applies the deltas
/// (REQ-L4-02).
#[test]
fn clinamen_applies_deltas() {
    let v = Verbum::new(0).with_conatus(100).with_onus(100).with_tranquillitas(30).with_ascensus(100);
    let m = Memoria::new(0).with_eventus(1).with_ramus(3).with_gradus(77);
    let (v1, m1) = fire(v.with_semen(0x1010), m);
    assert_eq!((m1.eventus(), m1.ramus(), m1.gradus()), (1, 1, 0));
    assert_eq!((v1.onus(), v1.conatus(), v1.tranquillitas()), (70, 100, 30));
    let (v2, m2) = fire(v.with_semen(0x2010), m);
    assert_eq!(m2.ramus(), 2);
    assert_eq!((v2.onus(), v2.conatus(), v2.tranquillitas()), (145, 90, 24));
    let (v0, _) = fire(v.with_semen(0x3010), m);
    assert_eq!((v0.onus(), v0.conatus(), v0.tranquillitas()), (105, 92, 34));
    let (vs, _) = fire(v.with_onus(240).with_tranquillitas(2).with_semen(0x2010), m);
    assert_eq!((vs.onus(), vs.tranquillitas()), (255, 0));
}

/// At 600 ticks a clinamen yields to another, never itself; special events end at 240 (REQ-L4-03, D5).
#[test]
fn clinamen_closes_and_rotates() {
    let m = Memoria::new(0).with_eventus(6).with_ramus(1).with_gradus(600);
    let m1 = close(Verbum::new(0).with_semen(0x0300), m);
    assert_eq!((m1.eventus(), m1.ramus(), m1.gradus()), (2, 3, 0));
    for s in 1..=0xFFFFu16 {
        assert_ne!(close(Verbum::new(0).with_semen(s), m).eventus(), 6);
    }
    let v = Verbum::new(0).with_semen(0x0500);
    for e in 8..=15u8 {
        let r = release(v, Memoria::new(0).with_eventus(e).with_ramus(0).with_gradus(240));
        assert_eq!((r.eventus(), r.ramus(), r.gradus()), (5, 3, 0));
    }
    let (_, m2, n) = event(Verbum::new(0).with_semen(0x0100), m.with_eventus(2).with_gradus(599), 0, 0, false);
    assert_eq!((m2.eventus(), m2.ramus(), m2.gradus(), n), (4, 3, 0, Nuntius::NIHIL));
    assert_eq!(age(Memoria::new(0).with_gradus(1023)).gradus(), 1023);
}

/// The ring shifts within 14 bits; ascent resets; weight falls, effort rises; affirmation
/// ends in serenity (REQ-L4-07).
#[test]
fn cycle_closure() {
    let v = Verbum::new(0).with_ascensus(255).with_onus(100).with_conatus(100).with_tranquillitas(10);
    let m = Memoria::new(0).with_clausurae(0x2003);
    let (v1, m1) = closure(v, m);
    assert_eq!(m1.clausurae(), 0x0006);
    assert_eq!((v1.ascensus(), v1.onus(), v1.conatus(), v1.tranquillitas()), (0, 92, 124, 10));
    assert_eq!(v1.modus(), Modus::Vigilia);
    let (va, _) = closure(v.with_modus(Modus::AmorFati), m);
    assert_eq!((va.tranquillitas(), va.modus()), (14, Modus::Vigilia));
    let (vs, _) = closure(v.with_onus(3).with_conatus(250).with_tranquillitas(62).with_modus(Modus::AmorFati), m);
    assert_eq!((vs.onus(), vs.conatus(), vs.tranquillitas()), (0, 255, 63));
}

/// The observer's return marks the present cycle (REQ-L4-08).
#[test]
fn interruption_marks_cycle() {
    let m = Memoria::new(0).with_clausurae(0b110);
    assert_eq!(interrupt(m).clausurae(), 0b111);
    assert_eq!(interrupt(interrupt(m)).clausurae(), 0b111);
}

/// Personae enter only in absence, through the shared gate, at their hour and place
/// (REQ-L4-09, D4).
#[test]
fn personae_conditions() {
    let v = Verbum::new(0).with_semen(0x0200).with_ascensus(128).with_onus(50).with_conatus(100);
    let m = Memoria::new(0).with_eventus(3).with_ramus(3);
    assert_eq!(persona(v, m, 128, 60, false), Some(DIOGENES));
    assert_eq!(persona(v, m, 128, 59, false), None);
    assert_eq!(persona(v, m, 128, 0, true), Some(DIOGENES));
    assert_eq!(persona(v.with_semen(0x0201), m, 128, 60, false), None);
    assert_eq!(persona(v, m.with_ramus(0), 128, 60, false), None);
    assert_eq!(persona(v, m, 111, 60, false), None);
    assert_eq!(persona(v.with_ascensus(100), m, 190, 60, false), Some(HERACLITUS));
    assert_eq!(persona(v.with_ascensus(129), m, 190, 60, false), None);
    assert_eq!(persona(v.with_ascensus(250), m, 31, 1800, false), Some(DAEMON));
    assert_eq!(persona(v.with_ascensus(250), m, 31, 1799, false), None);
    assert_eq!(persona(v.with_ascensus(250), m, 32, 1800, false), None);
    let (v13, m13) = visit(v.with_ascensus(20), m, DIOGENES);
    assert_eq!((v13.ascensus(), v13.onus()), (0, 58));
    assert_eq!((m13.eventus(), m13.ramus(), m13.gradus()), (DIOGENES, 0, 0));
    let (v14, _) = visit(v.with_curvatura(-10), m, HERACLITUS);
    assert_eq!((v14.onus(), v14.curvatura()), (34, -2));
    assert_eq!(visit(v.with_conatus(200), m, DAEMON).0.modus(), Modus::AmorFati);
    let (v15, _) = visit(v.with_conatus(199), m, DAEMON);
    assert_eq!((v15.modus(), v15.onus()), (Modus::Vigilia, 66));
}

/// Quies first; a stimulus wakes the dreamer; the dream needs night, calm, absence and
/// r ≥ r∞; affirmation holds; the dream persists without a stimulus (REQ-L4-10, D3).
#[test]
fn modus_precedence() {
    use Modus::{AmorFati, Quies, Somnium, Vigilia};
    let v = Verbum::new(0).with_semen(0x1000);
    let m = Memoria::new(0).with_eventus(2).with_ramus(3);
    for mo in [Vigilia, Quies, Somnium, AmorFati] {
        assert_eq!(modus_law(v.with_modus(mo), m, 38, 0, true, false, 0).0.modus(), Quies);
    }
    let dream = v.with_modus(Somnium);
    let (a, _, n) = modus_law(dream, m, 200, 16, true, false, 4000);
    assert_eq!((a.modus(), n), (Vigilia, Nuntius::AGNITIO));
    assert_eq!(modus_law(dream, m, 200, 0, false, false, 4000).2, Nuntius::AGNITIO);
    assert_eq!(modus_law(dream, m, 200, 0, true, true, 4000).2, Nuntius::AGNITIO);
    assert_eq!(modus_law(dream, m, 200, 15, true, false, 4000).0.modus(), Somnium);
    assert_eq!(modus_law(v, m, 200, 1, true, false, 2820).0.modus(), Somnium);
    assert_eq!(modus_law(v, m, 200, 1, true, false, 2819).0.modus(), Vigilia);
    assert_eq!(modus_law(v, m, 200, 2, true, false, 2820).0.modus(), Vigilia);
    assert_eq!(modus_law(v.with_semen(0x1001), m, 200, 1, true, false, 2820).0.modus(), Vigilia);
    assert_eq!(modus_law(v, m, 200, 1, false, false, u32::MAX).0.modus(), Vigilia);
    assert_eq!(modus_law(v.with_modus(AmorFati), m, 200, 255, false, true, 0).0.modus(), AmorFati);
    assert_eq!(modus_law(v.with_modus(Quies), m, 39, 0, true, false, 0).0.modus(), Vigilia);
}

/// The recognition: ascent and curvature reset, effort +32, event 12 (REQ-L4-11).
#[test]
fn agnitio_resets() {
    let dream = Verbum::new(0).with_modus(Modus::Somnium).with_ascensus(180).with_curvatura(-64).with_conatus(100).with_semen(7);
    let m = Memoria::new(0).with_eventus(4).with_ramus(1).with_gradus(300);
    let (v, m1) = agnitio(dream, m);
    assert_eq!((v.ascensus(), v.curvatura(), v.conatus(), v.modus()), (0, 0, 132, Modus::Vigilia));
    assert_eq!((m1.eventus(), m1.ramus(), m1.gradus()), (AGNITIO, 0, 0));
    assert_eq!(agnitio(dream.with_conatus(240), m).0.conatus(), 255);
}

/// Curvature per tick by mode (REQ-L4-11).
#[test]
fn curvature_dynamics() {
    use Modus::{AmorFati, Quies, Somnium, Vigilia};
    let c = |mo: Modus, k: i8| curvature(Verbum::new(0).with_modus(mo).with_curvatura(k)).curvatura();
    assert_eq!([c(Somnium, 0), c(Somnium, -62), c(Somnium, -64), c(Somnium, -128)], [-3, -64, -64, -64]);
    assert_eq!([c(AmorFati, 0), c(AmorFati, 63), c(AmorFati, 127)], [2, 64, 64]);
    assert_eq!([c(Vigilia, -64), c(Vigilia, 64), c(Vigilia, 1), c(Vigilia, -1), c(Quies, 40)], [-48, 48, 0, 0, 30]);
    let mut v = Verbum::new(0).with_curvatura(-64);
    for _ in 0..16 { v = curvature(v); }
    assert_eq!(v.curvatura(), 0);
}

/// r = 2.5 + 0.6·night + 0.6·min(idle/3600, 1); the dream needs r ≥ r∞ (REQ-L1-04).
#[test]
fn somnium_eligibility_threshold() {
    assert_eq!(crate::natura::logistic_r(false, 0), Fx(0x28000));
    assert_eq!(crate::natura::logistic_r(true, 3600), Fx(0x3B334));
    assert_eq!(crate::natura::logistic_r(true, u32::MAX), Fx(0x3B334));
    assert!(!somnium_eligible(true, 2819));
    assert!(somnium_eligible(true, 2820));
    assert!(!somnium_eligible(false, u32::MAX));
}

/// Contemplation: Quies, a persona, or sleep for 64 ticks after the recognition (D7).
#[test]
fn contemplation_states() {
    let v = Verbum::new(0);
    assert!(contemplating(v.with_modus(Modus::Quies), Memoria::new(0).with_ramus(3)));
    assert!(contemplating(v, Memoria::new(0).with_eventus(HERACLITUS)));
    assert!(contemplating(v, Memoria::new(0).with_eventus(AGNITIO).with_gradus(63)));
    assert!(!contemplating(v, Memoria::new(0).with_eventus(AGNITIO).with_gradus(64)));
    assert!(!contemplating(v, Memoria::new(0).with_eventus(3).with_ramus(1)));
}
```

- [ ] **Passo 2:** `cargo test -p sisyphium arbitrium` — falha.
- [ ] **Passo 3:** implementar as leis em `arbitrium/mod.rs` e `logistic_r` em `natura.rs`.
- [ ] **Passo 4:** `cargo test -p sisyphium` verde.
- [ ] **Passo 5:** reescrever os requisitos listados em **Arquivos**.
- [ ] **Passo 6:** `scripts/verify.sh`; commit `feat(arbitrium): add the laws of pace, effort, clinamina, personae, modus and closure`.

### Tarefa 3: `sensus` e `transumptio`

**Arquivos:** criar `crates/sisyphium/src/sensus.rs` e `crates/sisyphium/src/transumptio.rs`;
modificar `lib.rs`; `docs/03-especificacao.md` REQ-L7-03 (acrescenta D8).

**Interfaces:**
- Consome: `Verbum`, `Memoria`, `natura::mix`, `Nuntius`, `Modus`, constante `arbitrium::METALEPSIS`.
- Produz:
  - `sensus::occlusion(v: Verbum) -> Verbum` — `automatum` ⊕ 0xAA se `onus` > 128, senão rotação à esquerda de 1 bit (REQ-L6-01).
  - `sensus::liminal(automatism: u8, returned_ms: u32) -> bool` — ≥ 200 ∧ < 3000 (REQ-L6-02).
  - `sensus::rare(epoch_ms: i64, clausurae: u16, eventus: u8) -> bool` — janela w = `epoch_ms.div_euclid(4000)`; ctz(`mix(w as u64 ^ (clausurae as u64) ≪ 40 ^ (eventus as u64) ≪ 56)`) ≥ 13 (REQ-L6-03).
  - `sensus::quadrant_of(eventus: u8) -> u8` — (3·`eventus`) mod 4 (REQ-L6-04).
  - `transumptio::quadrant(x: u8, y: u8) -> u8` — ((y ≫ 7) ≪ 1) | (x ≫ 7) (REQ-L7-01).
  - `transumptio::weight(oraculum: u32, row: u8, col: u8) -> u8` — bits 8·row + 2·col.
  - `transumptio::oracle_update(oraculum: u32, row: u8, q: u8) -> u32` (REQ-L7-01).
  - `transumptio::trap(oraculum: u32, row: u8) -> Option<u8>` — primeiro índice do máximo quando máximo = 3 ∧ soma ≤ 5 (REQ-L7-02).
  - `transumptio::touch(v: Verbum, m: Memoria, x: u8, y: u8) -> (Verbum, Memoria, Nuntius)`: q = `quadrant(x, y)`; fora de Somnium, se `trap(oraculum, quadrans)` = Some(q) → `modus` AmorFati, `eventus` 11, `ramus` 0, `gradus` 0, linha `quadrans` = 0x55, `METALEPSIS`; senão `oracle_update`; por fim `quadrans` = q (REQ-L7-03, D8).
  - `transumptio::abyme(v: Verbum, u: u8, w: u8, z: u8) -> (bool, bool)` (REQ-L7-04).

- [ ] **Passo 1: testes que falham:**

```rust
// sensus.rs
/// ⊕ 0xAA under weight, rotation otherwise (REQ-L6-01).
#[test]
fn occlusion_mutation() {
    let v = Verbum::new(0).with_automatum(0b1000_0001);
    assert_eq!(occlusion(v.with_onus(129)).automatum(), 0b0010_1011);
    assert_eq!(occlusion(v.with_onus(128)).automatum(), 0b0000_0011);
}
/// Regular interaction just after a return (REQ-L6-02).
#[test]
fn liminal_anomaly_window() {
    assert!(liminal(200, 2999));
    assert!(!liminal(199, 0));
    assert!(!liminal(255, 3000));
    assert!(!liminal(255, u32::MAX));
}
/// About one window in 2¹³ opens; the gate holds for four seconds (REQ-L6-03).
#[test]
fn rare_gate_probability() {
    let hits = (0..(1i64 << 20)).filter(|w| rare(w * 4000, 0b101, 3)).count();
    assert!((90..=170).contains(&hits), "{hits}");
    for w in [0i64, 7, 1 << 30, -5] {
        for d in [0, 1, 3999] {
            assert_eq!(rare(w * 4000 + d, 1, 1), rare(w * 4000, 1, 1));
        }
    }
    let _ = (rare(i64::MIN, u16::MAX, 15), rare(i64::MAX, 0, 0));
}
/// The node of event k sits in quadrant 3k mod 4 (REQ-L6-04).
#[test]
fn inhibition_of_return_cycle() {
    assert_eq!([0u8, 1, 2, 3, 4, 5, 6, 7].map(quadrant_of), [0, 3, 2, 1, 0, 3, 2, 1]);
    for k in 0..16u8 {
        let mut seen = [false; 4];
        for j in 0..4u8 { seen[usize::from(quadrant_of((k + j) & 15))] = true; }
        assert!(seen.iter().all(|&s| s));
    }
}

// transumptio.rs
/// One touch raises its cell and, above a row sum of 6, lowers the others (REQ-L7-01).
#[test]
fn oracle_update() {
    let uniform = 0x5555_5555u32;
    let o = super::oracle_update(uniform, 2, 1);
    assert_eq!(weight(o, 2, 1), 2);
    assert_eq!(o & !0x00FF_0000, uniform & !0x00FF_0000);
    let o = super::oracle_update(o, 2, 1);
    assert_eq!([0, 1, 2, 3].map(|c| weight(o, 2, c)), [1, 3, 1, 1]);
    let o = super::oracle_update(o, 2, 1);
    assert_eq!(weight(o, 2, 1), 3);
    let o = super::oracle_update(o, 2, 0);
    assert_eq!([0, 1, 2, 3].map(|c| weight(o, 2, c)), [2, 2, 0, 0]);
    assert_eq!([quadrant(0, 0), quadrant(127, 127), quadrant(128, 0), quadrant(0, 128), quadrant(255, 255)], [0, 0, 1, 2, 3]);
}
/// Ready when the row's maximum is 3 and its sum at most 5 (REQ-L7-02).
#[test]
fn trap_ready_condition() {
    let row = |w: [u32; 4]| w[0] | w[1] << 2 | w[2] << 4 | w[3] << 6;
    assert_eq!(trap(row([3, 1, 1, 0]) << 8, 1), Some(0));
    assert_eq!(trap(row([3, 1, 1, 1]) << 8, 1), None);
    assert_eq!(trap(row([2, 2, 1, 0]) << 8, 1), None);
    assert_eq!(trap(row([1, 3, 0, 0]) << 8, 1), Some(1));
    assert_eq!(trap(row([3, 0, 0, 0]), 0), Some(0));
    assert_eq!(trap(0x5555_5555, 0), None);
}
/// The predicted touch inverts agency; elsewhere the oracle learns; the dreamer is never
/// prophesied (REQ-L7-03, D8).
#[test]
fn metalepsis_fires() {
    let v = Verbum::new(0);
    let o = (0x5555_5555u32 & !0xFF00) | (0x07 << 8);
    let m = Memoria::new(0).with_oraculum(o).with_quadrans(1).with_eventus(5).with_ramus(3);
    let (v1, m1, n) = touch(v, m, 10, 10);
    assert_eq!(n, Nuntius::METALEPSIS);
    assert_eq!((v1.modus(), m1.eventus(), m1.ramus(), m1.gradus()), (Modus::AmorFati, 11, 0, 0));
    assert_eq!((m1.oraculum() >> 8) & 0xFF, 0x55);
    assert_eq!(m1.quadrans(), 0);
    let (v2, m2, n2) = touch(v, m, 200, 10);
    assert_eq!((n2, v2.modus(), m2.quadrans()), (Nuntius::NIHIL, Modus::Vigilia, 1));
    assert_eq!(weight(m2.oraculum(), 1, 1), 2);
    let (_, m3, n3) = touch(v.with_modus(Modus::Somnium), m, 10, 10);
    assert_eq!((n3, m3.eventus(), m3.quadrans()), (Nuntius::NIHIL, 5, 0));
}
/// Macro bit from the word; micro bit from the rotated and mixed word at zoom z/16 (REQ-L7-04).
#[test]
fn abyme_sampling() {
    let v = Verbum::new(0x0123_4567_89AB_CDEF);
    let micro = v.bits().rotate_left(13) ^ 0x9E37_79B9_7F4A_7C15;
    for u in 0..8u8 { for w in 0..8u8 { for z in [0u8, 16, 37, 255] {
        let (a, b) = abyme(v, u, w, z);
        assert_eq!(a, (v.bits() >> (8 * u32::from(w) + u32::from(u))) & 1 == 1);
        let (u2, w2) = ((u32::from(u) * u32::from(z)) >> 4 & 7, (u32::from(w) * u32::from(z)) >> 4 & 7);
        assert_eq!(b, (micro >> (8 * w2 + u2)) & 1 == 1);
    }}}
    assert_eq!(abyme(Verbum::new(1), 0, 0, 16).0, true);
}
```

- [ ] **Passo 2:** `cargo test -p sisyphium sensus transumptio` — falha.
- [ ] **Passo 3:** implementar; registrar em `lib.rs`.
- [ ] **Passo 4:** `cargo test -p sisyphium` verde.
- [ ] **Passo 5:** `scripts/verify.sh`; commit `feat(sensus): add occlusion, liminal anomaly, rare gate and inhibition of return` e commit `feat(transumptio): add the oracle, the trap, the metalepsis and the abyme`.

### Tarefa 4: `mundus::step` e `mundus::advance`

**Arquivos:** modificar `crates/sisyphium/src/mundus.rs`; `docs/03-especificacao.md` (REQ-H-06
com a assinatura de D1); `docs/04-arquitetura.md` (§4 protocolo com `step(&facultates,
&indicia)`; §6 ABI: `sisyphium_step(uint64_t *verbum, uint64_t *memoria, const SyFacultates *f,
const SyIndicia *i)` e `sisyphium_advance(uint64_t *verbum, uint64_t *memoria, const
SyFacultates *f, const SyIndicia *i, uint32_t ticks, uint32_t tick_ms)`; nova §7.1 "Ordem do
tick" com a sequência abaixo).

**Interfaces:**
- Consome: Tarefas 1–3; `ephemeris::ephemeris`; `corpus::chip`; `natura::galois`.
- Produz: `Mundus::step(&mut self, facultates: &Facultates, indicia: &Indicia) -> Nuntius`;
  `Mundus::advance(&mut self, facultates: &Facultates, indicia: &Indicia, ticks: u32, tick_ms: u32) -> Nuntius`.

Ordem do tick (D2), com `eph = ephemeris(epoch_ms, tz_offset_min, australis)`:
1. `semen` ← `galois(semen)`.
2. `ascensus` = 255 → `closure`; `CLAUSURA`.
3. `leaving` → `interrupt`.
4. `occluded` → `occlusion`.
5. `chip(verbum, seism)` = Some → aplica; `LAPSUS`.
6. `touch` = Some((x, y)) → `transumptio::touch`; pode devolver `METALEPSIS`.
7. `modus_law(…, energy, seism, eph.nocturnal, touch.is_some(), idle_s)`; pode devolver `AGNITIO`.
8. `curvature`.
9. `stepped` = ¬`contemplating` ∧ `paces`; com passo, `ascensus` +1.
10. `continuous(verbum, stepped, contemplating, seism, idle_s)`.
11. `event(verbum, memoria, eph.phase, idle_s, occluded)`; pode devolver `CLINAMEN` ou `PERSONA`.
12. `rare(epoch_ms, clausurae, eventus)` → `RARUM`.

`advance`: início = `epoch_ms` − `ticks`·`tick_ms` (saturante em `i64`); o tick k ∈ 1..=`ticks`
usa `epoch_ms` = início + k·`tick_ms` (saturante); os demais indicia constantes; devolve a união
dos `Nuntius`.

- [ ] **Passo 1: testes que falham** em `mundus.rs`:

```rust
/// Every world, under eight environments, steps without panic; the generator advances by one;
/// operational curvature stays operational; only the seven announcement bits appear (REQ-L0-05).
#[test]
fn step_is_total() {
    let f = Facultates::default();
    let envs: [Indicia; 8] = environments();   // the eight vectors listed below
    let mut s = 1u16;
    for _ in 0..65_535u32 {
        s = galois(s);
        let k = mix(u64::from(s));
        let world = Mundus { verbum: Verbum::new(k), memoria: Memoria::new(mix(k)) };
        for i in &envs {
            let mut w = world;
            let n = w.step(&f, i);
            assert_eq!(w.verbum.semen(), galois(world.verbum.semen()));
            if world.verbum.curvatura().unsigned_abs() <= 64 {
                assert!(w.verbum.curvatura().unsigned_abs() <= 64);
            }
            assert_eq!(n.0 & 0x80, 0);
        }
    }
}
```
Os oito ambientes de `environments()` (auxiliar documentado): (1) `Indicia::default()`; (2)
`epoch_ms` 1 782 054 000 000, energia 120, ociosidade 3 600; (3) `epoch_ms` 1 797 831 000 000,
energia 200, ociosidade 4 000; (4) energia 20; (5) sismo 255, automatismo 255, `returned_ms` 0;
(6) toque `Some((0, 255))`, ociosidade 0; (7) `occluded` e `leaving`; (8) `epoch_ms` `i64::MIN`,
energia 0, ociosidade 0.

```rust
/// `advance` is `ticks` steps at interpolated epochs and returns the union of announcements
/// (REQ-H-06).
#[test]
fn advance_equals_repeated_step() {
    let f = Facultates::default();
    let i = Indicia { epoch_ms: 1_782_054_000_000, energy: 255, idle_s: 3600, ..Indicia::default() };
    for ticks in [0u32, 1, 7, 4096] {
        let mut a = Mundus::genesis(1);
        let na = a.advance(&f, &i, ticks, 42);
        let mut b = Mundus::genesis(1);
        let mut nb = Nuntius::NIHIL;
        for k in 1..=ticks {
            let at = Indicia { epoch_ms: i.epoch_ms - i64::from(ticks) * 42 + i64::from(k) * 42, ..i };
            nb = nb.cum(b.step(&f, &at));
        }
        assert_eq!((a, na), (b, nb), "ticks {ticks}");
    }
}

/// Epochs saturate at the ends of time (review focus 1).
#[test]
fn advance_saturates_epoch() {
    let f = Facultates::default();
    for epoch in [i64::MIN + 5, i64::MAX - 5] {
        let mut w = Mundus::genesis(9);
        let _ = w.advance(&f, &Indicia { epoch_ms: epoch, ..Indicia::default() }, 65_536, 60_000);
        assert_ne!(w, Mundus::genesis(9));
    }
}

/// The summit rests one tick, then the cycle closes; a touch wakes the dreamer; Quies holds
/// the figure still (D2, D7, D8).
#[test]
fn step_order() {
    let f = Facultates::default();
    let i = Indicia { epoch_ms: 1_782_054_000_000, ..Indicia::default() };
    let mut top = Mundus::genesis(5);
    top.verbum = top.verbum.with_ascensus(255);
    let n = top.step(&f, &i);
    assert!(n.contains(Nuntius::CLAUSURA));
    assert!(top.verbum.ascensus() <= 1);
    let mut dream = Mundus::genesis(6);
    dream.verbum = dream.verbum.with_modus(Modus::Somnium).with_ascensus(100);
    let night = Indicia { epoch_ms: 1_797_831_000_000, touch: Some((10, 10)), idle_s: 4000, ..Indicia::default() };
    assert!(dream.step(&f, &night).contains(Nuntius::AGNITIO));
    assert_eq!((dream.memoria.eventus(), dream.verbum.ascensus()), (12, 0));
    let mut still = Mundus::genesis(7);
    still.verbum = still.verbum.with_modus(Modus::Quies).with_ascensus(77);
    let dark = Indicia { energy: 20, ..i };
    let _ = still.advance(&f, &dark, 10_000, 60_000);
    assert_eq!(still.verbum.ascensus(), 77);
}
```

- [ ] **Passo 2:** `cargo test -p sisyphium mundus` — falha.
- [ ] **Passo 3:** implementar `step` e `advance` na ordem acima; `mundus` só compõe leis.
- [ ] **Passo 4:** `cargo test -p sisyphium` verde; medir `advance` de 65 536 ticks em release
  (`cargo test --release -p sisyphium -- --ignored metire_advance`, teste ignorado que imprime a
  mediana de 11 execuções) e registrar o número no relatório.
- [ ] **Passo 5:** reescrever os documentos listados em **Arquivos**.
- [ ] **Passo 6:** `scripts/verify.sh`; commit `feat(mundus): step and advance the world in the order of the tick`.

### Tarefa 5: `consilium`

**Arquivos:** criar `crates/sisyphium/src/consilium.rs`; modificar `lib.rs`;
`docs/06-consilium.md` (§3 área do Musivum; §4 bônus de AmorFati e máximo por situação; §5
estado inicial, piso, tabela EXP, aceitação); `docs/04-arquitetura.md` §2 (dependências de
`consilium` e de `concentus`, que lê `Regimen` de `hospes` e não depende de `consilium`).

**Interfaces:**
- Consome: `Facultates`, `Mundus`, `Regimen`, `Stilus`, constantes de camada de `hospes`,
  `natura::galois`, `spatium::compositio` (escala s), `memoria`.
- Produz:
  - `pub const fn band_tick_ms(energy: u8) -> u32` — ≥ 179 → 42; 77..=178 → 125; 39..=76 → 250; < 39 → 60 000 (REQ-L1-07).
  - `pub const INTERVALS: [u32; 5] = [42, 125, 250, 1000, 60_000];`
  - `pub const EXP: u128 = 0x0507_090C_1014_1A22_2C38_495D_789A_C6FF;` (Apêndice B).
  - `pub fn budget(facultates: &Facultates, energy: u8) -> u32` — `compute`·`energy`/64.
  - `pub fn cost(regimen: &Regimen, facultates: &Facultates) -> u32` — unidades/s (§3; saturante).
  - `pub fn value(regimen: &Regimen, mundus: &Mundus, facultates: &Facultates, energy: u8) -> u32` — §4.
  - `pub fn value_max(mundus: &Mundus, facultates: &Facultates, energy: u8) -> u32` — soma dos máximos por dimensão sob os forçamentos (D12).
  - `pub fn floor(facultates: &Facultates, mundus: &Mundus, energy: u8) -> Regimen` — stilus forçado, senão Musivum; `strata` 1 (2 em Somnium); `layers` 0; `lines` 0; `interval_ms` 60 000; `tick_ms` pela faixa (60 000 em Quies).
  - `pub fn consilium(facultates: &Facultates, mundus: &Mundus, energy: u8) -> Regimen`.

Custo por quadro em unidades: base 10; +3 por estrato além do primeiro; CAELUM 6; MARGO 5;
ANNALES 1 por linha; PERSONAE 2; RUINAE 1; IMPOSSIBILIA 4; HEXAGRAMMA 1; PUNCTA 4; Littera +2;
Bicolor +3. Custo/s = unidades · área · 1000 / (57 600 · `interval_ms`) em `u64`, área = W·H, ou
(W/s)·(H/s) em Musivum. Valor (§4): céu 8 (×2 com `ramus` ≠ 3); moldura 5 (×2 com `onus` > 128);
crônicas 2 por linha (×2 com `ramus` ≠ 3); personae 6 só com `eventus` 13..=15; ruínas 3 só com
popcount(`clausurae` ≫ 1) > 0; impossibilia 7 só com (`conatus` > 200 ∧ `onus` > 200) ∨ Somnium;
hexagrama 2 (×2 com `tranquillitas` ≥ 48); pontilhado 2 (×2 em Somnium); estratos 2 por estrato
além do primeiro (×2 em Somnium); cadência por `interval_ms` 42 → 6 (0 com energia < 179), 125 →
4, 250 → 2, 1000 → 1, 60 000 → 0; stilus Creta 4 (8 em AmorFati), Musivum 3, Littera 2 (6 na
redução eidética: energia 39..=76 ∧ `tranquillitas` ≥ 48), Bicolor 1 (6 em Quies).

Forçamentos (REQ-C-04), nesta precedência: `cells` → Littera; `color_bits` = 1 → Bicolor;
Quies → Bicolor e `tick_ms` 60 000; Somnium → `strata` ≥ 2 e stilus ≠ Littera (Littera vira
Creta), salvo `cells`; `max_hz` = 0 → `interval_ms` 60 000. Piso de intervalo: o menor valor de
`INTERVALS` ≥ max(`tick_ms`, ⌈1000/`max_hz`⌉).

Busca (determinística; g começa em `semen` do mundo):

```text
base ← Creta, strata 4, tick_ms pela faixa, interval_ms = piso, lines 6,
       layers = camadas com valor > 0 na situação; aplicar forçamentos
inicial ← base se cost(base) ≤ budget, senão floor
se cost(floor) > budget: devolver floor
score(r) = 8·value(r) − cost(r)                      (i64; λ = 1/8)
cur ← best ← inicial; T ← energy/16
para it em 0..64:
    se value(best) ≥ value_max·4/5: parar
    g ← galois(g); dim ← g mod 5; g ← galois(g); pick ← g ≫ 4
    p ← cur com a dimensão dim trocada: 0 stilus [Creta, Musivum, Littera, Bicolor][pick mod 4];
        1 strata 1 + pick mod 4; 2 interval INTERVALS[pick mod 5] elevado ao piso;
        3 layers ⊕ (1 ≪ (pick mod 8)); 4 lines pick mod 7; aplicar forçamentos
    se p = cur ou cost(p) > budget: continuar
    Δ ← score(cur) − score(p)
    aceitar se Δ ≤ 0; senão, se T > 0: g ← galois(g); aceitar se (g ∧ 0xFF) < EXP[min(15, 4·Δ/T)]
    se aceito: cur ← p; se score(p) > score(best): best ← p
devolver best
```

- [ ] **Passo 1: testes que falham:**

```rust
/// Energy bands fix the world's cadence (REQ-L1-07).
#[test]
fn regimen_energy_bands() {
    let cases = [(255, 42), (179, 42), (178, 125), (77, 125), (76, 250), (39, 250), (38, 60_000), (0, 60_000)];
    for (e, t) in cases { assert_eq!(band_tick_ms(e), t, "energy {e}"); }
    let r = consilium(&Facultates::default(), &Mundus::genesis(1), 20);
    assert_eq!((r.stilus, r.tick_ms), (Stilus::Bicolor, 60_000));
}

/// Forcings in their precedence (REQ-C-04).
#[test]
fn consilium_forcings() {
    let w = Mundus::genesis(1);
    let cells = Facultates { cells: true, color_bits: 1, ..Facultates::default() };
    assert_eq!(consilium(&cells, &w, 255).stilus, Stilus::Littera);
    let mono = Facultates { color_bits: 1, ..Facultates::default() };
    assert_eq!(consilium(&mono, &w, 255).stilus, Stilus::Bicolor);
    let mut quies = w; quies.verbum = quies.verbum.with_modus(Modus::Quies);
    let q = consilium(&Facultates::default(), &quies, 255);
    assert_eq!((q.stilus, q.tick_ms, q.interval_ms), (Stilus::Bicolor, 60_000, 60_000));
    let mut dream = w; dream.verbum = dream.verbum.with_modus(Modus::Somnium);
    let d = consilium(&Facultates::default(), &dream, 200);
    assert!(d.strata >= 2 && d.stilus != Stilus::Littera);
    assert_eq!(consilium(&Facultates { cells: true, ..Facultates::default() }, &dream, 200).stilus, Stilus::Littera);
    let still = consilium(&Facultates { max_hz: 0, ..Facultates::default() }, &w, 255);
    assert_eq!((still.interval_ms, still.tick_ms), (60_000, 42));
    let mut amor = w; amor.verbum = amor.verbum.with_modus(Modus::AmorFati);
    assert_eq!(consilium(&Facultates::default(), &amor, 255).stilus, Stilus::Creta);
}

/// Same inputs, same regimen (REQ-C-03).
#[test]
fn consilium_deterministic() {
    let f = Facultates { width: 1920, height: 1080, ..Facultates::default() };
    for seed in [1u64, 2, 0xBEEF] {
        let w = Mundus::genesis(seed);
        assert_eq!(consilium(&f, &w, 150), consilium(&f, &w, 150));
    }
}

/// The chosen regimen fits the budget, or is the floor when nothing fits; the presentation
/// interval never undercuts the host's rate (REQ-C-03, review focus 4).
#[test]
fn consilium_respects_budget() {
    for (w, h) in [(272u16, 60u16), (320, 180), (640, 360), (1920, 1080)] {
        for energy in [0u8, 38, 39, 76, 77, 178, 179, 255] {
            for compute in [0u8, 96, 255] {
                for max_hz in [0u8, 1, 24, 60] {
                    let f = Facultates { width: w, height: h, compute, max_hz, ..Facultates::default() };
                    let world = Mundus::genesis(u64::from(w) ^ u64::from(energy) << 16);
                    let r = consilium(&f, &world, energy);
                    assert!(cost(&r, &f) <= budget(&f, energy) || r == floor(&f, &world, energy));
                    assert!(r.interval_ms >= r.tick_ms);
                    if max_hz == 0 { assert!(r.interval_ms >= 60_000); }
                    else { assert!(r.interval_ms >= 1000u32.div_ceil(u32::from(max_hz))); }
                }
            }
        }
    }
}

/// A plentiful host reaches 80 % of the situation's maximum (REQ-C-03).
#[test]
fn consilium_reaches_aspiration() {
    let f = Facultates::default();
    let w = Mundus::genesis(1);
    let r = consilium(&f, &w, 255);
    assert!(value(&r, &w, &f, 255) * 5 >= value_max(&w, &f, 255) * 4);
    assert!(cost(&r, &f) <= budget(&f, 255));
}
```

- [ ] **Passo 2:** `cargo test -p sisyphium consilium` — falha.
- [ ] **Passo 3:** implementar; `EXP` lido por `(EXP >> (8·k)) & 0xFF`.
- [ ] **Passo 4:** `cargo test -p sisyphium` verde.
- [ ] **Passo 5:** reescrever `docs/06-consilium.md` como em **Arquivos**.
- [ ] **Passo 6:** `scripts/verify.sh`; commit `feat(consilium): choose the regimen by satisficing search`.

### Tarefa 6: texto do estado, grafo do céu, aforismo, espaço negativo, contraste e som

**Arquivos:** modificar `crates/sisyphium/src/concentus.rs` (registro por `annales`, D17, D18),
`crates/sisyphium/src/mundus.rs` (`render`, `render_cellae`, `palette`),
`crates/sisyphium/src/signum/caelum.rs` (grafo, máscaras), `crates/sisyphium/src/signum/scaena.rs`
(composição do céu; tons do fundo por `contrastus`), `docs/03-especificacao.md` (REQ-L5-06 e
REQ-L5-10 com as regras abaixo; REQ-L2-06 com D20), `docs/04-arquitetura.md` §5 (aforismo no
passo 2; plinto fora da projeção), `docs/05-estetica.md` §3 (frase-resumo da tabela de tons, que
erra nas colunas da encosta) e §10 (contraste e índice sonoro como implementados).

**Interfaces:**
- Consome: `arbitrium::{annales, Annales, revealed}`; `sensus::quadrant_of`;
  `caelum::sky_region` (checkpoint 2, harmonia aplicada); `natura::mix`; `spatium::{scatter64,
  isqrt}`; `scaena::render`; `scaena::render_cellae` (devolve `bool` desde o checkpoint 2).
- Produz:
  - `Mundus::render(&self, facultates: &Facultates, indicia: &Indicia, regimen: &Regimen, tab: &mut Tabula)` — linhas de `annales(memoria, lingua, leaving)`.
  - `Mundus::render_cellae(&self, facultates: &Facultates, indicia: &Indicia, regimen: &Regimen, scratch: &mut Tabula, cells: &mut Cellae) -> bool`.
  - `Mundus::palette(&self, facultates: &Facultates, indicia: &Indicia, regimen: &Regimen) -> [[u8; 3]; 16]`.
  - `caelum::MASKS: [[u16; 16]; 3]` (Apêndice C; bit 15 = coluna 0).
  - `caelum::snap(p: (i32, i32), square: Rect, mask: &[u16; 16]) -> (i32, i32)`.
  - `caelum::node(sky: Rect, eventus: u8, ramus: u8, i: u8, onus: u8) -> (i32, i32)` — posição do
    nó i pela lei abaixo, com o encaixe na máscara quando `onus` > 200.
  - `caelum::graph(pictor: &mut Pictor, sky: Rect, trunk: (i32, i32), eventus: u8, ramus: u8, shown: u8, onus: u8, s: u16, lum_edge: u8, lum_node: u8) -> Option<(i32, i32)>` — devolve o nó 0.

Leis do grafo (REQ-L5-06, L5-10, L6-04, D15):
- Tronco em (m, ⌊H·φ⁻²/2⌋). Quadrante q = `quadrant_of(eventus)`: sub-retângulo do céu
  x = sky.x + (q ∧ 1)·⌊sky.w/2⌋, y = sky.y + (q ≫ 1)·⌊sky.h/2⌋, w = ⌊sky.w/2⌋, h = ⌊sky.h/2⌋.
- Nó i < `shown`: k = `mix(eventus | ramus ≪ 8 | i ≪ 16)`; posição = (qx + `scatter64(k, 16)`·qw ≫ 16,
  qy + `scatter64(k ≫ 32, 16)`·qh ≫ 16). Com `onus` > 200: `snap` sobre `MASKS[eventus mod 3]` no
  quadrado de lado L = max(1, min(qw, qh)) centrado no quadrante.
- `snap`: (u, v) = ((px − x0)·16/L, (py − y0)·16/L) limitados a 0..=15; célula de contorno (célula
  cheia com vizinho de 4 vazio ou fora da grade) de menor distância de Manhattan, empate pelo
  menor v·16 + u; resultado = (x0 + (2u + 1)·L/32, y0 + (2v + 1)·L/32).
- Arestas (Bresenham, espessura s, estrato 2): nó 0 ao tronco; ramus 0: nó i ao nó i − 1; ramus 1
  (bifurcação): nó 1 ao nó 0, nó i ≥ 2 ao nó i − 2; ramus 2 (fractal): nó i ao nó i − 1 e, para i ≥ 1,
  dois sub-ramos do nó i a nó_i + ((dx − dy)/2, (dx + dy)/2) e nó_i + ((dx + dy)/2, (dy − dx)/2), com
  (dx, dy) = nó_i − nó_{i−1}.
- Nó: quadrado de 2s × 2s em `lum_node`; aresta em `lum_edge`.
- Aforismo: `annales(…).aphorismus` em `text` com canto em (nó0.x + 2s, nó0.y − 5s), escala s,
  luminância 12, estrato 2.
- `shown` = `revealed(gradus)` com `eventus` 0..=7 ∧ `ramus` 0..=2 e camada CAELUM; 0 nos demais.
- Composição do céu (§5 passo 2): constelação, grafo, aforismo, hexagrama, lua, na região
  `sky_region(comp, W, onus)`.
- D17: `concentus.contrastus` = 15 em Quies; o tom de cada traço da encosta e das ruínas é
  limitado a 15 − `contrastus`.
- D18: `concentus.sonus` = ⌊255·8/√(64 + `onus`)⌋ com `isqrt`; 255 com `onus` 0, decrescente.
- D20: o texto do plinto é desenhado com `Pictor::flat` (sem curvatura); a encosta e as ruínas
  não escrevem nas linhas do plinto (recorte em y < S).

- [ ] **Passo 1: testes que falham:**

```rust
// concentus.rs — substitui registrum_by_gradus
/// The register of the text is the register of its last visible line (D13).
#[test]
fn registrum_by_annales() { /* for (gradus, reg) in [(0, Intimum), (24, Intimum), (72, Vivum),
    (144, Civile), (240, Epochale), (360, Abstractum)] with eventus 3, ramus 1: concentus(..).registrum
    == reg; eventus 12 gradus 30 → Intimum, gradus 80 → Abstractum; eventus 11 gradus 30 →
    Abstractum; ramus 3 and leaving → the interruption register (Epochale or Civile) */ }

// caelum.rs
/// One node per revealed line inside the event's quadrant; edges follow the branch (REQ-L5-06,
/// REQ-L6-04).
#[test]
fn sky_graph_nodes() { /* sky = compositio(320, 180, 6).sky; for eventus in 0..8, ramus in 0..3,
    i in 0..5: node(sky, eventus, ramus, i, 0) lies in the quadrant rectangle of
    quadrant_of(eventus) and is deterministic; on a blank 320×180 tabula, graph(..) with shown 0
    draws nothing and returns None; with shown n ≥ 1 it returns Some(node(.., 0, ..)) and every
    pixel of lum 14 lies within s of some node(.., i < n, ..); ramus 1 with shown 3 draws no
    pixel of lum 9 on the segment from node 2 to node 1 that is absent from the segment from
    node 2 to node 0 (bifurcation) */ }

/// Above a burden of 200 every node sits on the contour of the event's mask (REQ-L5-10).
#[test]
fn negative_space_mask_snap() { /* for eventus 0..3 (masks 0, 1, 2) and 64 random points: the
    snapped point maps back to a contour cell of MASKS[e % 3]; with onus 200 the graph equals the
    unsnapped graph; snap of a point already on a contour cell centre is the identity */ }

/// A degenerate sky draws within the frame (review focus 5).
#[test]
fn degenerate_sky() { /* render via Mundus::render at 272×60 lines 6 and at 1×1 with a clinamen
    at gradus 400 and onus 230: no panic; guard bytes after the prefix intact; with a scene of
    height 0 (figure height 0) no actor pixel is drawn: the plinth rows equal those of the same
    frame rendered with the figure, boulder and shadow skipped; the prefix guard also holds in
    Musivum at 640×360 (s = 2) and 1920×1080 (s = 6) */ }

/// Lower energy, darker ground around the figure; Quies is the darkest (D17).
#[test]
fn contrast_governs_ground() { /* same world at energy 255, 120, 60 and in Quies: the maximum
    luminance among slope and ruin pixels is ≤ 15 − contrastus in each frame, and contrastus is
    8, 10, 12 and 15 */ }

/// The chronicle stays straight while the world dreams (D20).
#[test]
fn plinth_outside_dream() { /* the same world rendered with curvature 0 and −64: inside the
    plinth's text rectangle the pixels of luminance 13 are identical; above the plinth the frames
    differ; no slope or ruin pixel lies in rows ≥ S */ }

/// Accented letters reach the cells and the corrupted harmony reaches the frame (wiring).
#[test]
fn render_cellae_keeps_accents() { /* render_cellae with the line "AÇÃO" → the first four
    cells of the text row hold glyphs 0, 81, 73, 14 */ }

/// The corrupted harmony reaches the composed frame (REQ-L5-12).
#[test]
fn harmony_reaches_scene() { /* the same world with onus 200 and 201 (nothing else changed):
    the sky rows of the two frames differ */ }

/// Pitch falls with the square root of the mass (D18).
#[test]
fn sonus_by_mass() { /* sonus(onus 0) = 255; sonus(onus 255) = 2040/isqrt(319) = 2040/17 = 120;
    non-increasing over onus 0..=255 */ }

// mundus.rs
/// The world renders its own text (REQ-L4-04): a chronicle at gradus 300 writes luminance 13 in
/// the plinth; an awaiting world writes none there.
#[test]
fn render_uses_annales() { /* … */ }
```

- [ ] **Passo 2:** `cargo test -p sisyphium` — falha nos testes novos.
- [ ] **Passo 3:** implementar; `concentus` lê `annales` e aplica D17 e D18; `scaena` compõe o
  céu com `graph` e aforismo e limita os tons do fundo.
- [ ] **Passo 4:** `cargo test -p sisyphium` — `golden_frames` falha onde a forma mudou: gerar os
  PPM (`dump_golden_ppm`), converter em PNG, inspecionar, atualizar `GOLDEN` no mesmo commit e
  anexar ao relatório os caminhos dos PNG antes e depois.
- [ ] **Passo 5:** reescrever REQ-L5-06, REQ-L5-10, `04-arquitetura.md` §5 e `05-estetica.md` §10.
- [ ] **Passo 6:** `scripts/verify.sh`; commit `feat(signum): draw the world's own text, the sky graph and negative space`.

### Tarefa 7: personae, ensō, raio raro, sombra liminar, impossibilia, kintsugi e abismo

**Arquivos:** criar `crates/sisyphium/src/signum/figurae.rs`; modificar `signum/mod.rs`,
`signum/raster.rs` (`Pictor::edge`), `corpus.rs` (`draw_boulder` com micro),
`signum/scaena.rs`, `docs/04-arquitetura.md` §5 (passo 5 com personae, ensō, impossibilia,
kintsugi; abismo na rocha em contemplação), `docs/05-estetica.md` §6 (atributos das personae),
`docs/07-narrativa.md` §5 (atributos), `docs/03-especificacao.md` REQ-L3-05 ("o rosto DEVE ser
um bloco sólido de lado max(1, round(4·h/24)) px, sem traços") e REQ-L2-06 (D19).

**Interfaces:**
- Consome: `arbitrium::{contemplating, asleep, DIOGENES, HERACLITUS, DAEMON}`;
  `sensus::{liminal, rare}`; `transumptio::{trap, abyme}`; `spatium::{penrose, sin, cos}`;
  `natura::{henon, mix}`; `caelum::ray`; `corpus::{draw_figure, Figura}`.
- Produz:
  - `Pictor::edge(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, lum: u8, stratum: u8, label: u8, modo: Modo)` — Bresenham de espessura 1 com rótulo.
  - `Pictor::plot` sob κ > 0 segue D19 (REQ-L2-06 reescrito com a cobertura pelas projeções
    vizinhas).
  - `figurae::enso(pictor: &mut Pictor, center: (i32, i32), r: i32, start: u16, thick: u8, s: u16, lum: u8)` — 28 de 32 segmentos de 2048 unidades de ângulo a partir de `start`; os 4 últimos com espessura s.
  - `figurae::quadrant_center(width: u16, height: u16, q: u8) -> (i32, i32)` — (W·(1 + 2·(q ∧ 1))/4, H·(1 + 2·(q ≫ 1))/4).
  - `figurae::staircase(pictor: &mut Pictor, center: (i32, i32), span: u32, k: i32, lum: u8)` — 4·span pontos de `penrose(t, 0, 0, span)` escalados por k, unidos por `edge` em `Modo::Dominans`, classe de `penrose`, rótulo = (t / span) mod 3.
  - `figurae::henon_point(window: i64, sky: Rect) -> (i32, i32)` — 32 + (`mix(window as u64)` ∧ 31) iterações de `henon` a partir de (0, 0); x ∈ [−1,5; 1,5] e y ∈ [−0,5; 0,5] mapeados linearmente na região do céu e limitados a ela.
  - `figurae::seam(pictor: &mut Pictor, center: (i32, i32), r: i32, key: u64, s: u16, lum: u8)` — quatro pontos (cx − r, cx − r/3, cx + r/3, cx + r) com desvios verticais ((key ≫ 8j) ∧ 0xFF) mod (r/2 + 1) − r/4, unidos por linhas de espessura s.
  - `draw_boulder(pictor: &mut Pictor, center: (i32, i32), d: u16, scale: u16, texture: u64, tones: [u8; 3], stratum: u8, seed_rows: u8, micro: Option<(Verbum, u8)>)` —
    `seed_rows` é a máscara das linhas da textura desenhadas com tons 7 (aceso) e 5 (apagado)
    (D16; o compositor passa `0b11u8.rotate_left(ascensus & 7)`); com `micro` = Some((v, z)) e
    `scale` ≥ 8, cada célula recebe a subgrade 8×8 de `abyme(v, a, b, z).1` (a, b = posição na
    célula·8/scale): macro aceso → 15/12, macro apagado → 6/4 (micro aceso/apagado).

Regras de cena (passo 5 de §5):
- Persona ativa (`eventus` 13..=15) com camada PERSONAE: Diógenes = figura Erecta de luminância
  13 no ponto da encosta `ascensus` + 40 (saturante), lâmpada = disco de raio s, luminância 15,
  centrado em (cabeça.x + lado + 2s, cabeça.y + 2·lado), com (cabeça.x, cabeça.y, lado) devolvidos
  por `draw_figure`; Heráclito = figura Contemplans de luminância 11 em `ascensus` − 48 (saturante) e
  pira = triângulo de altura `figure_h`/2 no pé da encosta, ápice deslocado s nas fases ímpares da
  marcha, luminância 14; Daemon = figura Erecta de luminância 4 no cume (`ascensus` 255).
- A figura principal usa `arbitrium::contemplating(verbum, memoria)` na postura (substitui o
  predicado local de persona do checkpoint 2).
- D16: na textura da rocha, as duas linhas que contêm `semen` (linhas k e k + 1 mod 8, k =
  `ascensus` mod 8, depois da rotação de REQ-L3-03) usam tom 7 (bit aceso) e 5 (apagado).
- Ensō quando `trap(oraculum, quadrans)` = Some(q): centro `quadrant_center(W, H, q)`, raio
  `figure_h`, `start` = `quadrans`·16 384, espessura `linea`·s, luminância 14, estrato 1.
- Raio raro quando `rare(epoch_ms, clausurae, eventus)`: `ray` de `henon_point(epoch_ms.div_euclid(4000),
  céu)` ao topo da cabeça da figura, luminância 15.
- Sombra liminar: com `liminal(automatism, returned_ms)`, a sombra vai para a direita do pé.
- Impossibilia com camada IMPOSSIBILIA ∧ ((`conatus` > 200 ∧ `onus` > 200) ∨ Somnium):
  `staircase` com span 6, k = 2s, centro (cume.x − 10k, cume.y − 6k), luminância 11.
- Kintsugi em AmorFati: `seam` sobre a rocha com key = `mix(verbum ≫ 48)`, luminância 15.
- Abismo: `micro` = Some((verbum, 16 + (`gradus` ∧ 15))) quando `contemplating`.

- [ ] **Passo 1: testes que falham** (em `figurae.rs` e `scaena.rs`):

```rust
/// The circle of one stroke: 28 of 32 segments, open at the end (REQ-L7-02).
#[test]
fn enso_open_circle() { /* blank 200×200; enso at (100,100), r 40, start 0, thick 1, s 1, lum 14:
    pixels of lum 14 exist at distance 38..=42 from the centre on ≥ 85 % of 64 sampled angles and
    none on the 4 sampled angles inside the gap */ }

/// The predicted quadrant holds the ensō and only it (REQ-L7-02).
#[test]
fn enso_in_predicted_quadrant() { /* world with row quadrans = [3,1,0,0] → q = 0: render minus the
    same world with uniform oracle differs only inside quadrant 0's rectangle */ }

/// The last flight passes behind the first; labels change along the stair (REQ-L2-08, L2-10).
#[test]
fn impossibilia_dominance() { /* blank tabula; staircase span 6 k 2: every pixel drawn by both
    flight 0 and flight 3 keeps class 0; labels 0, 1 and 2 all present */ }

/// The shadow flips in the liminal window (REQ-L6-02).
#[test]
fn liminal_shadow_flips() { /* render with automatism 255, returned_ms 0 vs 0/u32::MAX: shadow
    pixels (lum 2) left of the foot in the second, right in the first */ }

/// A rare window draws the ray; any other window does not (REQ-L6-03).
#[test]
fn rare_ray_drawn() { /* search w from 0 until rare(w·4000, c, e); henon_point lies in the sky;
    the composer's ray step draws ≥ 1 pixel of lum 15 in that window and none in window w + 1
    (when rare(w+1) is false) */ }

/// Gold seam only in affirmation (kintsugi).
#[test]
fn kintsugi_seam_in_amor_fati() { /* interior of the boulder (radius − 2) holds lum 15 only in
    AmorFati */ }

/// Under the smallest expansion a one-pixel stroke stays one pixel; coverage stays dense (D19).
#[test]
fn expansion_keeps_stroke_width() { /* 320×180: a horizontal line of thickness 1 across the
    frame at y = 40 under κ = 1 lights exactly one pixel per column it reaches; under κ = 64 at
    most 3; curvature_density still ≥ 0.95 for κ ∈ {1, 64} */ }

/// The two rows of the seed glitter below the differential threshold (D16).
#[test]
fn semen_rows_dimmed() { /* boulder drawn on a blank tabula for ascensus 0 and 3: pixels of the
    rows holding semen carry only tones 7 and 5 (and the rim 15); other rows carry 12 and 4 */ }

/// In contemplation the boulder contains the word within the word (REQ-L7-04).
#[test]
fn abyme_in_contemplation() { /* 1920×1080 frame: interior lums of the boulder ⊇ {15, 12, 6, 4}
    in Quies; ⊆ {12, 4, 15} in Vigilia */ }

/// Each persona leaves its figure and attribute; without the layer, nothing (REQ-L4-09).
#[test]
fn personae_drawn() { /* eventus 13/14/15 at gradus 10 vs eventus 3 ramus 3: frames differ;
    with layers without PERSONAE: frames equal */ }
```

- [ ] **Passo 2:** `cargo test -p sisyphium` — falha nos testes novos.
- [ ] **Passo 3:** implementar `figurae`, `Pictor::edge`, `draw_boulder` com micro, e a cena.
- [ ] **Passo 4:** `cargo test -p sisyphium`; hashes-ouro alterados: PPM → PNG, inspeção, `GOLDEN`
  atualizado no mesmo commit; caminhos dos PNG no relatório.
- [ ] **Passo 5:** reescrever os documentos listados em **Arquivos**.
- [ ] **Passo 6:** `scripts/verify.sh`; commit `feat(signum): draw personae, enso, rare ray, impossible stair, kintsugi and abyme`.

### Tarefa 8: quadros-ouro de §7, medições e fechamento

**Arquivos:** criar `crates/sisyphium/tests/aurea.rs`; modificar `signum/scaena.rs` (remove
`GOLDEN`, `golden_frames`, `golden_scenarios`, `dump_golden_ppm` e `metire_frames`, que passam a
`aurea.rs`), `docs/08-verificacao.md` (§1 "Onde": U e P em `#[cfg(test)]` do módulo, G em
`crates/sisyphium/tests/aurea.rs`; §7 tabela com todos os cenários), `README.md` ("Checkpoint
presente: 3 (escolha, percepção e narração: `arbitrium` com corpus em duas línguas, `sensus`,
`transumptio`, `consilium`; `mundus::step`, `advance` e `render`)"); remover este plano.

**Interfaces:**
- Consome: só a API pública (`Mundus::{genesis, advance, render}`, `Facultates`, `Indicia`,
  `Regimen`, `Tabula`, `scaena::fnv1a`).
- Produz: testes `golden_frames`, `sky_graph_golden`, `night_golden`, `agnitio_golden`; ignorados
  `dump_golden_ppm` (escreve PPM em `$CARGO_TARGET_TMPDIR`) e `metire_frames` (mediana e p95 por
  stilus a 320×180 com 1000 quadros e 1920×1080 com 100, e `advance` de 65 536 ticks).

Cenários (todos com `Facultates::default()` salvo dimensões):

| Teste | Mundo | Indicia | Regimen |
| --- | --- | --- | --- |
| `golden_frames` vigilia | `genesis(1)` + `advance` 4 096 ticks de 42 ms terminando em 1 782 054 000 000 | energia 255, sismo 0 | Creta, 4 estratos, 42 ms, todas as camadas, 6 linhas |
| `golden_frames` acedia | `genesis(2)` + 4 096 ticks de 125 ms até 1 790 112 600 000 (2026-09-22 18:30 −03:00); depois `onus` 160 | energia 120 | Musivum, 2 estratos, 125 ms |
| `golden_frames` somnium | `genesis(3)` com Somnium e curvatura −48 | 1 797 831 000 000, energia 200, ociosidade 3 600 | Creta, 4 estratos |
| `golden_frames` quies | `genesis(4)` | 1 774 008 000 000 (2026-03-20 09:00 −03:00), energia 20 | Bicolor, 1 estrato, 60 000 ms |
| `sky_graph_golden` | `genesis(1)`, `eventus` 3, `ramus` 0, 1 e 2, `gradus` 400; quarto quadro com `ramus` 2 e `onus` 230 | 1 782 054 000 000 | padrão |
| `night_golden` | `genesis(5)`, `ascensus` 60 | 1 798 090 200 000 (2026-12-24 02:30 −03:00, lua cheia), energia 200 | padrão |
| `agnitio_golden` | `genesis(6)`, `eventus` 12, `ramus` 0, `gradus` 100 | 1 782 054 000 000 | padrão |

- [ ] **Passo 1:** mover os auxiliares para `aurea.rs`, escrever os cenários com hashes 0 e rodar
  `cargo test -p sisyphium --test aurea -- --ignored dump_golden_ppm`; converter em PNG.
- [ ] **Passo 2:** inspecionar cada PNG (encosta, figura, rocha, céu, texto legível, lua, grafo
  com o aforismo, máscaras, dígitos da agnitio) e registrar os hashes. Os dígitos da agnitio
  ficam a pelo menos 2s da moldura direita (x = W − m − 96s − 2s); corrigir em `compose_text` se
  tocarem.
- [ ] **Passo 3:** `cargo test --release -p sisyphium --test aurea -- --ignored metire_frames
  --nocapture`; registrar mediana e p95 no relatório (REQ-H-05: Creta ≤ 2 ms, Musivum ≤ 1 ms,
  Littera ≤ 1 ms, Bicolor ≤ 2 ms a 320×180; ≤ 30 ms a 1920×1080).
- [ ] **Passo 4:** reescrever `08-verificacao.md` §1 e §7 e o `README.md`; remover este plano.
- [ ] **Passo 5:** `scripts/verify.sh`; commits `test(mundus): pin golden frames to the scenarios of the verification` e `docs: close checkpoint 3`.

---

## Apêndice A — corpus

Linha `e.r.i` = `eventus`, `ramus`, linha. Inglês à esquerda, português à direita. Os módulos
`annales_en` e `annales_pt` transcrevem estas linhas exatamente.

Settings das consequências (uma civilização por ramus): 0 Atenas (séc. V a.C.), Chang'an (séc.
VIII), Tenochtitlan (séc. XV); 1 Troia (séc. XII a.C.), Persépolis (séc. V a.C.), Crotona (séc. VI
a.C.); 2 Pataliputra (séc. III a.C.), Benin (séc. XVI), Timbuktu (séc. XIV); 3 Cuzco (séc. XV),
Alexandria (séc. III a.C.), Edo (séc. XVII); 4 Uruk (séc. XXX a.C.), Cartago (séc. III a.C.), Kyoto
(séc. XII); 5 Nálanda (séc. VII), Axum (séc. IV), Rapa Nui (séc. XIII); 6 Constantinopla (séc.
VI), Bagdá (séc. IX), Teotihuacan (séc. IV); 7 Córdoba (séc. X), Tonga (séc. XIII), Angkor (séc.
XII).

| Linha | Inglês | Português |
| --- | --- | --- |
| 0.0.0 | THE LEAF SKIDS INTO THE DUST. | A FOLHA DESLIZA PARA A POEIRA. |
| 0.0.1 | AN OWL WAKES IN THE OLIVE TREE. | UMA CORUJA ACORDA NA OLIVEIRA. |
| 0.0.2 | THE FLIGHT TIPS A LAMP IN ATHENS. | O VOO DERRUBA UMA LÂMPADA EM ATENAS. |
| 0.0.3 | THE SCROLLS ON MERCY BURN TONIGHT. | OS ROLOS SOBRE A CLEMÊNCIA ARDEM. |
| 0.0.4 | A WORD FOR MERCY IS LOST FOR AGES. | FALTA UMA PALAVRA PARA CLEMÊNCIA. |
| 0.1.0 | THE LEAF CRACKS UNDER THE SANDAL. | A FOLHA ESTALA SOB A SANDÁLIA. |
| 0.1.1 | A CRICKET LEAPS FROM THE SOUND. | UM GRILO SALTA COM O SOM. |
| 0.1.2 | A SILK CART SWERVES NEAR CHANG'AN. | UM CARRO DE SEDA DESVIA EM CHANG'AN. |
| 0.1.3 | THE DYE FOR THE IMPERIAL ROBE SPILLS. | A TINTA DO MANTO IMPERIAL ENTORNA. |
| 0.1.4 | A DYNASTY DATES A FALL FROM THAT YEAR. | UMA DINASTIA DATA A QUEDA DESSE ANO. |
| 0.2.0 | THE LEAF STAYS WHOLE ON THE STONE. | A FOLHA FICA INTEIRA NA PEDRA. |
| 0.2.1 | AN ANT CARRIES THE LEAF TO THE NEST. | UMA FORMIGA LEVA A FOLHA AO NINHO. |
| 0.2.2 | THE SOIL OF A CHINAMPA HOLDS. | O SOLO DE UMA CHINAMPA RESISTE. |
| 0.2.3 | MAIZE FEEDS A LAKE CITY ONE MORE YEAR. | O MILHO NUTRE A CIDADE DO LAGO UM ANO. |
| 0.2.4 | THE ORDER OF THINGS STAYS UNMOVED. | A ORDEM DAS COISAS FICA IMÓVEL. |
| 1.0.0 | THE PEBBLE GRINDS THE HEEL AT EACH STEP. | O SEIXO MÓI O CALCANHAR A CADA PASSO. |
| 1.0.1 | THE PEBBLE POLISHES AND ROLLS OUT. | O SEIXO POLIDO ROLA PARA FORA. |
| 1.0.2 | A TROUT TURNS AWAY. A HOOK WAITS. | UMA TRUTA DESVIA. UM ANZOL ESPERA. |
| 1.0.3 | THE FISHER LOCKS THE GATES OF TROY. | O PESCADOR FECHA AS PORTAS DE TROIA. |
| 1.0.4 | NO LINE OF AENEAS. NO ROME. | SEM LINHAGEM DE ENEIAS. SEM ROMA. |
| 1.1.0 | THE SANDAL EMPTIES. THE STONE FALLS. | A SANDÁLIA ESVAZIA. A PEDRA CAI. |
| 1.1.1 | THE STONE STRIKES A HIVE ON THE LEDGE. | A PEDRA BATE NUMA COLMEIA DA ESCARPA. |
| 1.1.2 | THE SWARM SCATTERS A PURPLE CARAVAN. | O ENXAME DISPERSA A CARAVANA DE PÚRPURA. |
| 1.1.3 | NO DYE FOR THE ROBE AT PERSEPOLIS. | SEM TINTA PARA O MANTO EM PERSÉPOLIS. |
| 1.1.4 | A GENERAL READS THE DELAY AS WEAKNESS. | UM GENERAL LÊ O ATRASO COMO FRAQUEZA. |
| 1.2.0 | THE HEEL GRINDS THE STONE TO DUST. | O CALCANHAR MÓI A PEDRA EM PÓ. |
| 1.2.1 | A WARM GUST LIFTS THE DUST SKYWARD. | UMA RAJADA QUENTE ERGUE O PÓ AO CÉU. |
| 1.2.2 | HAIL FALLS ON THE BARLEY OF CROTON. | CAI GRANIZO NA CEVADA DE CROTONA. |
| 1.2.3 | THE SCHOOL OF NUMBERS GOES HUNGRY. | A ESCOLA DOS NÚMEROS PASSA FOME. |
| 1.2.4 | THE INFINITE WAITS TWELVE CENTURIES. | O INFINITO ESPERA DOZE SÉCULOS. |
| 2.0.0 | THE SHADOW CROSSES THE STONE. | A SOMBRA CRUZA A PEDRA. |
| 2.0.1 | A KITE CIRCLES ONCE AND LEAVES. | UM MILHAFRE GIRA UMA VEZ E VAI. |
| 2.0.2 | A SCRIBE IN PATALIPUTRA LOOKS UP TOO. | UM ESCRIBA EM PATALIPUTRA TAMBÉM OLHA. |
| 2.0.3 | THE EDICT ON MERCY IS COPIED TWICE. | O ÉDITO DA CLEMÊNCIA SAI EM DUAS CÓPIAS. |
| 2.0.4 | A SECOND COPY OUTLIVES THE FIRE. | A SEGUNDA CÓPIA SOBREVIVE AO FOGO. |
| 2.1.0 | THE SHADOW PASSES UNSEEN. | A SOMBRA PASSA SEM OLHAR. |
| 2.1.1 | THE BIRD LANDS ON A SMELTER'S ROOF. | A AVE POUSA NO TELHADO DE UM FUNDIDOR. |
| 2.1.2 | THE BRONZE COOLS A BREATH TOO SOON. | O BRONZE ESFRIA UM SOPRO ANTES. |
| 2.1.3 | A PLAQUE OF BENIN KEEPS A CRACK. | UMA PLACA DO BENIM GUARDA UMA RACHA. |
| 2.1.4 | THE CRACK IS READ AS A SIGN FOR AGES. | A RACHA É LIDA COMO SINAL POR ERAS. |
| 2.2.0 | THE STONE SLIPS A HAND'S WIDTH. | A PEDRA ESCORREGA UM PALMO. |
| 2.2.1 | A LIZARD DARTS FROM THE CRACK. | UM LAGARTO FOGE DA FENDA. |
| 2.2.2 | A CAMEL BALKS AT THE GATE OF TIMBUKTU. | UM CAMELO EMPACA À PORTA DE TIMBUKTU. |
| 2.2.3 | A BALE OF SALT ROTS AT THE WALL. | UM FARDO DE SAL APODRECE NO MURO. |
| 2.2.4 | A SCHOOL CLOSES FOR LACK OF SALT. | UMA ESCOLA FECHA POR FALTA DE SAL. |
| 3.0.0 | THE DROP IS WIPED BEFORE THE FALL. | A GOTA É ENXUTA ANTES DE CAIR. |
| 3.0.1 | THE STONE STAYS DRY AND LIGHT. | A PEDRA FICA SECA E LEVE. |
| 3.0.2 | A KNOT IS TIED ON A QUIPU IN CUZCO. | UM NÓ É ATADO NUM QUIPU EM CUZCO. |
| 3.0.3 | THE COUNT OF MAIZE COMES OUT EXACT. | A CONTA DO MILHO SAI EXATA. |
| 3.0.4 | NO FAMINE IS WRITTEN FOR THAT YEAR. | NENHUMA FOME É ESCRITA NESSE ANO. |
| 3.1.0 | THE DROP DARKENS A SPOT OF STONE. | A GOTA ESCURECE UM PONTO DA PEDRA. |
| 3.1.1 | A BEE TASTES THE SALT AND LEAVES. | UMA ABELHA PROVA O SAL E VAI. |
| 3.1.2 | A GARDEN OF ALEXANDRIA LOSES THE BEES. | UM JARDIM DE ALEXANDRIA PERDE ABELHAS. |
| 3.1.3 | THE FIGS OF THE LIBRARY DO NOT RIPEN. | OS FIGOS DA BIBLIOTECA NÃO AMADURECEM. |
| 3.1.4 | A COPYIST GOES HUNGRY. A BOOK WAITS. | UM COPISTA PASSA FOME. UM LIVRO ESPERA. |
| 3.2.0 | THE SALT WAKES THE TONGUE. | O SAL ACORDA A LÍNGUA. |
| 3.2.1 | THE STEP QUICKENS FOR A WHILE. | O PASSO ACELERA POR UM TEMPO. |
| 3.2.2 | A SHIP LEAVES EDO ON THE MORNING TIDE. | UM NAVIO DEIXA EDO NA MARÉ DA MANHÃ. |
| 3.2.3 | A CHEST OF TEA REACHES THE NORTH. | UM BAÚ DE CHÁ CHEGA AO NORTE. |
| 3.2.4 | A HABIT OF TEA SPREADS ACROSS AN AGE. | UM HÁBITO DE CHÁ ATRAVESSA UMA ERA. |
| 4.0.0 | THE FOOT TESTS THE CRACK AND HOLDS. | O PÉ TESTA A RACHA E AGUENTA. |
| 4.0.1 | A BEETLE CRAWLS OUT OF THE SPLIT. | UM BESOURO SAI DA FENDA. |
| 4.0.2 | A POTTER IN URUK PRESSES A BEETLE SIGN. | UM OLEIRO DE URUK IMPRIME O BESOURO. |
| 4.0.3 | THE SIGN COUNTS JARS, THEN YEARS. | O SINAL CONTA JARROS, DEPOIS ANOS. |
| 4.0.4 | WRITING BEGINS WITH A CRACK. | A ESCRITA COMEÇA NUMA RACHA. |
| 4.1.0 | THE FOOT SKIPS THE CRACK. | O PÉ PULA A RACHA. |
| 4.1.1 | THE DUST SETTLES ON THE CRACK UNTOUCHED. | A POEIRA ASSENTA NA RACHA INTOCADA. |
| 4.1.2 | A SURVEYOR OF CARTHAGE READS AN OMEN. | UM AGRIMENSOR DE CARTAGO LÊ PRESSÁGIO. |
| 4.1.3 | THE HARBOR WALL IS BUILT A YEAR LATE. | O MURO DO PORTO SAI UM ANO ATRASADO. |
| 4.1.4 | A FLEET SAILS AGAINST AN OPEN CITY. | UMA FROTA SURGE DIANTE DA CIDADE ABERTA. |
| 4.2.0 | THE HEEL WIDENS THE CRACK TO A SPLIT. | O CALCANHAR ALARGA A RACHA EM FENDA. |
| 4.2.1 | THE SPLIT DRINKS THE FIRST RAIN. | A FENDA BEBE A PRIMEIRA CHUVA. |
| 4.2.2 | IN KYOTO A CISTERN STAYS EMPTY. | EM KYOTO UMA CISTERNA FICA VAZIA. |
| 4.2.3 | A TEMPLE BURNS WITH NO WATER NEAR. | UM TEMPLO ARDE SEM ÁGUA POR PERTO. |
| 4.2.4 | THE SUTRAS GO UP WITH THE ROOF. | OS SUTRAS SOBEM COM O TELHADO. |
| 5.0.0 | THE FOOT WAITS FOR THE ANT TO PASS. | O PÉ ESPERA A FORMIGA PASSAR. |
| 5.0.1 | THE SEED REACHES THE NEST. | A SEMENTE CHEGA AO NINHO. |
| 5.0.2 | A FIG SHADES A COURTYARD AT NALANDA. | A FIGUEIRA SOMBREIA UM PÁTIO EM NALANDA. |
| 5.0.3 | A MONK READS ONE PAGE MORE IN THE SHADE. | UM MONGE LÊ MAIS UMA PÁGINA À SOMBRA. |
| 5.0.4 | A PROOF ABOUT ZERO SURVIVES THE RAINS. | UMA PROVA SOBRE O ZERO VENCE AS CHUVAS. |
| 5.1.0 | THE FOOT CIRCLES THE ANT. | O PÉ CONTORNA A FORMIGA. |
| 5.1.1 | THE ANT CHANGES ROAD AND DROPS THE SEED. | A FORMIGA MUDA DE RUMO E LARGA O GRÃO. |
| 5.1.2 | A SEEDLING GROWS BESIDE A ROAD TO AXUM. | UMA MUDA CRESCE NO CAMINHO DE AXUM. |
| 5.1.3 | CARAVANS REST THERE A CENTURY LATER. | CARAVANAS PARAM ALI UM SÉCULO DEPOIS. |
| 5.1.4 | A MARKET KEEPS THE NAME OF A TREE. | UM MERCADO GUARDA O NOME DE UMA ÁRVORE. |
| 5.2.0 | THE ANT DIES UNDER A BLIND STEP. | A FORMIGA MORRE SOB UM PASSO CEGO. |
| 5.2.1 | THE SEED NEVER SPROUTS. | A SEMENTE NUNCA BROTA. |
| 5.2.2 | ON RAPA NUI ONE PALM FEWER TAKES ROOT. | EM RAPA NUI UMA PALMEIRA A MENOS CRESCE. |
| 5.2.3 | THE LAST CANOE WOOD RUNS OUT SOONER. | A MADEIRA DA ÚLTIMA CANOA ACABA ANTES. |
| 5.2.4 | AN ISLAND FORGETS THE SEA. | UMA ILHA ESQUECE O MAR. |
| 6.0.0 | A SHOUT GOES DOWN INTO THE VALLEY. | UM GRITO DESCE AO VALE. |
| 6.0.1 | A GOAT STARTLES AND DROPS A BELL. | UMA CABRA FOGE E PERDE O SINO. |
| 6.0.2 | A SHEPHERD'S BOY FINDS A ROMAN COIN. | UM MENINO PASTOR ACHA UMA MOEDA ROMANA. |
| 6.0.3 | THE COIN BUYS A PLACE IN A SCHOOL. | A MOEDA PAGA UM LUGAR NUMA ESCOLA. |
| 6.0.4 | A CODE OF LAWS OUTLASTS THE EMPIRE. | UM CÓDIGO DE LEIS SOBREVIVE AO IMPÉRIO. |
| 6.1.0 | THE ECHO DIES UNANSWERED. | O ECO MORRE SEM RESPOSTA. |
| 6.1.1 | A HARE KEEPS STILL IN THE GRASS. | UMA LEBRE FICA QUIETA NO CAPIM. |
| 6.1.2 | A HUNTER OF BAGHDAD COMES BACK EMPTY. | UM CAÇADOR DE BAGDÁ VOLTA SEM NADA. |
| 6.1.3 | THE EVENING GOES TO A PROBLEM INSTEAD. | A NOITE VAI PARA UM PROBLEMA. |
| 6.1.4 | ALGEBRA GAINS ONE NIGHT. | A ÁLGEBRA GANHA UMA NOITE. |
| 6.2.0 | THE ECHO COMES BACK CHANGED. | O ECO VOLTA MUDADO. |
| 6.2.1 | A SECOND VOICE HIDES IN THE SOUND. | UMA SEGUNDA VOZ VEM DENTRO DO SOM. |
| 6.2.2 | AT TEOTIHUACAN A FLUTE FINDS A NEW TONE. | EM TEOTIHUACAN UMA FLAUTA ACHA UM TOM. |
| 6.2.3 | THE TONE OPENS A FEAST EACH SPRING. | O TOM ABRE UMA FESTA A CADA PRIMAVERA. |
| 6.2.4 | A CITY KEEPS TIME BY A SOUND. | UMA CIDADE MARCA O TEMPO POR UM SOM. |
| 7.0.0 | THE BODY LEANS INTO THE GUST. | O CORPO PENDE CONTRA A RAJADA. |
| 7.0.1 | A SEED OF THISTLE FLIES PAST THE EAR. | UMA SEMENTE DE CARDO PASSA PELA ORELHA. |
| 7.0.2 | A THISTLE TAKES ROOT IN CÓRDOBA. | UM CARDO CRIA RAIZ EM CÓRDOBA. |
| 7.0.3 | A GARDENER LEARNS A GRAFT THAT YEAR. | UM JARDINEIRO APRENDE UM ENXERTO. |
| 7.0.4 | ORANGES REACH A COLD COUNTRY. | LARANJAS CHEGAM A UM PAÍS FRIO. |
| 7.1.0 | THE CLIMB WAITS FOR THE WIND TO PASS. | A SUBIDA ESPERA O VENTO PASSAR. |
| 7.1.1 | A GULL RIDES THE GUST OUT TO SEA. | UMA GAIVOTA VAI NA RAJADA PARA O MAR. |
| 7.1.2 | A CANOE OF TONGA READS THE BIRD'S LINE. | UMA CANOA DE TONGA LÊ A LINHA DA AVE. |
| 7.1.3 | THE CANOE FINDS AN ISLAND BY THE GULL. | A CANOA ACHA UMA ILHA PELA GAIVOTA. |
| 7.1.4 | A ROUTE IS SUNG FOR TWENTY GENERATIONS. | UMA ROTA É CANTADA POR VINTE GERAÇÕES. |
| 7.2.0 | THE FACE TURNS FROM THE DUST. | O ROSTO DESVIA DA POEIRA. |
| 7.2.1 | A MOTH SHELTERS IN THE COLLAR. | UMA MARIPOSA ENTRA NO COLARINHO. |
| 7.2.2 | A MOTH RIDES A BARGE BOUND FOR ANGKOR. | A MARIPOSA VAI NUMA BARCAÇA PARA ANGKOR. |
| 7.2.3 | A SILK HARVEST FAILS ONE SEASON. | UMA SAFRA DE SEDA FALHA UMA ESTAÇÃO. |
| 7.2.4 | A TEMPLE LOSES A DECADE. | UM TEMPLO PERDE UMA DÉCADA. |
| aforismo 0 | □LEAF → ◊FIRE | □FOLHA → ◊FOGO |
| aforismo 1 | □STONE → ◊ROME | □PEDRA → ◊ROMA |
| aforismo 2 | □SHADOW → ◊SIGN | □SOMBRA → ◊SINAL |
| aforismo 3 | □DROP → ◊COUNT | □GOTA → ◊CONTA |
| aforismo 4 | □CRACK → ◊WORD | □RACHA → ◊LETRA |
| aforismo 5 | □SEED → ◊TREE | □GRÃO → ◊ÁRVORE |
| aforismo 6 | □ECHO → ◊LAW | □ECO → ◊LEI |
| aforismo 7 | □WIND → ◊ROUTE | □VENTO → ◊ROTA |
| persona 13 Diógenes | WHY CARRY A STONE WITH NO OWNER? | POR QUE CARREGAR UMA PEDRA SEM DONO? |
| persona 14 Heráclito | THE SAME STONE DOES NOT RISE TWICE. | A MESMA PEDRA NÃO SOBE DUAS VEZES. |
| persona 15 daemon | THIS CLIMB AGAIN, AND AGAIN, FOREVER? | ESTA SUBIDA DE NOVO, E DE NOVO, SEM FIM? |
| agnitio 0 | THE HEAD RESTS ON THE COLD STONE. | A CABEÇA REPOUSA NA PEDRA FRIA. |
| agnitio 1 | THE CLIMB IS A DREAM. THE STONE IS NOT. | A SUBIDA É SONHO. A PEDRA NÃO. |
| metalepse | THE LOOP RUNS THE HAND. | O CICLO MOVE A MÃO. |
| interrupção 0 | THE HAND MOVES. THE WAR WAITS. | A MÃO MEXE. A GUERRA ESPERA. |
| interrupção 1 | A CURSOR STOPS A CENTURY. | UM CURSOR DETÉM UM SÉCULO. |
| interrupção 2 | A GATE OF TROY STAYS HALF SHUT. | UMA PORTA DE TROIA FICA ENTREABERTA. |
| interrupção 3 | A CARAVAN CAMPS AT A SEALED WELL. | UMA CARAVANA ACAMPA NUM POÇO LACRADO. |

## Apêndice B — tabela EXP

EXP[k] = ⌊255·e^(−k/4)⌋, k = 0..=15: 255, 198, 154, 120, 93, 73, 56, 44, 34, 26, 20, 16, 12, 9,
7, 5; empacotada em `u128` com o byte k na posição 8·k: `0x0507_090C_1014_1A22_2C38_495D_789A_C6FF`.
Índice: k = min(15, ⌊4·Δ/T⌋).

## Apêndice C — máscaras de espaço negativo

Linhas de cima para baixo; bit 15 = coluna 0.

| Máscara | `eventus` mod 3 | Linhas |
| --- | --- | --- |
| ampulheta | 0 | `0x3FFC, 0x3FFC, 0x1FF8, 0x0FF0, 0x07E0, 0x03C0, 0x0180, 0x0180, 0x0180, 0x0180, 0x03C0, 0x07E0, 0x0FF0, 0x1FF8, 0x3FFC, 0x3FFC` |
| olho | 1 | `0x0000, 0x0000, 0x0000, 0x03C0, 0x1FF8, 0x7FFE, 0xFC3F, 0xF81F, 0xF81F, 0xFC3F, 0x7FFE, 0x1FF8, 0x03C0, 0x0000, 0x0000, 0x0000` |
| balança | 2 | `0x0180, 0x7FFE, 0x4182, 0x4182, 0xA185, 0xA185, 0xE187, 0x0180, 0x0180, 0x0180, 0x0180, 0x0180, 0x0180, 0x0180, 0x07E0, 0x0FF0` |
