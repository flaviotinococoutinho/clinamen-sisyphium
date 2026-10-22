# 04 — Arquitetura

## 1. Workspace

```
clinamen-sisyphium/
  Cargo.toml                 workspace
  crates/
    sisyphium/               núcleo: no_std, sem alloc, sem dependências, sem unsafe, sem ponto flutuante
    sisyphium-tabula/      hospedeiro: linha de comando; escreve PPM e PNG (codificador próprio)
    sisyphium-textus/      hospedeiro: terminal; Littera nativo; raster por meio-bloco
    sisyphium-limen/       hospedeiro: ABI C (staticlib, cdylib); exportação wasm
  hosts/
    macos/                   script de papel de parede + launchd
    linux/                   script de papel de parede (feh ou swww) + systemd timer
    windows/                 script de papel de parede (PowerShell) + agendador
    tela/                    página estática + wasm
  docs/
```

## 2. Módulos do núcleo

| Módulo | Nível | Conteúdo | Depende de |
| --- | --- | --- | --- |
| `verbum` | L0 | `Verbum`: leiaute, acesso a campos, empacotamento | — |
| `memoria` | L0 | `Memoria`: leiaute, acesso a campos, empacotamento | — |
| `natura` | L1 | gerador de Galois; regra 30; ruído 1/f sem estado; mapa logístico; atrator de Hénon; vento | `spatium` (ponto fixo) |
| `ephemeris` | L1 | fases circadiana, sazonal e lunar; noturno; festum | — |
| `consilium` | L1 | `Regimen`; busca satisfatória por recozimento | `hospes`, `natura` |
| `spatium` | L2 | Q16.16; seno de Bhaskara; isqrt; CORDIC; dispersão; projeção de curvatura; estratos; dominância; dimétrica; escada de Penrose; composição | — |
| `corpus` | L3 | marcha; postura; figuras de blocos; rocha como leitura do `Verbum`; lasca | `verbum`, `spatium`, `signum::raster`, `signum::pixel` |
| `arbitrium` | L4 | clinamina; annales (corpus en/pt); passo; leis de onus, conatus, tranquillitas, ascensus; personae; modus; agnitio; curvatura; contemplação; fecho; interrupção | `verbum`, `memoria`, `natura`, `hospes` |
| `concentus` | L5 | `Concentus`: vetor de afinação de todos os canais (tom, contraste, espessura, cadência, registro do texto, curvatura, índice sonoro) derivado do estado, da efeméride e do regimen | `verbum`, `ephemeris`, `consilium` |
| `signum` | L5 | pixel; paleta; stili; fonte; primitivas (linha, disco, retângulo, texto); chicote; grafo do céu; hexagrama; lua; ruínas; máscaras de espaço negativo; compositor de cena | `spatium`, `corpus`, `arbitrium`, `concentus` |
| `sensus` | L6 | mutação em oclusão; anomalia liminar; porta de evento raro; inibição de retorno | `verbum`, `natura` |
| `transumptio` | L7 | oráculo; armadilha; metalepse; amostragem em abismo | `memoria`, `verbum` |
| `hospes` | limiar | `Facultates`, `Indicia`, `Tabula`, `Cellae`, `Lingua`, `Nuntius` | — |
| `mundus` | — | `Mundus`: genesis, step, render, render_cellae, format, parse | todos |

Regra de dependência: `concentus` e `signum` não alteram estado; `arbitrium`, `sensus` e
`transumptio` alteram estado e não desenham; `mundus` ordena.

## 3. Estado

### 3.1 `Verbum` (64 bits; bit 0 = menos significativo)

| Bits | Campo | Tipo | Domínio | Lei |
| --- | --- | --- | --- | --- |
| 0..=15 | `semen` | u16 | 1..=65 535 | gerador de Galois `0xB400`; nunca zero |
| 16..=23 | `conatus` | u8 | 0..=255 | REQ-L4-06 |
| 24..=31 | `onus` | u8 | 0..=255 | REQ-L4-02, L4-06, L4-07, L4-09 |
| 32..=39 | `ascensus` | u8 | 0..=255 | REQ-L4-05, L4-07 |
| 40..=47 | `curvatura` | i8 | −64..=64 em operação; qualquer valor é válido | REQ-L4-11 |
| 48..=55 | `automatum` | u8 | anel de 8 células | REQ-L1-02, L6-01 |
| 56..=61 | `tranquillitas` | u6 | 0..=63 | REQ-L4-06; hexagrama |
| 62..=63 | `modus` | u2 | 0 Vigilia, 1 Quies, 2 Somnium, 3 AmorFati | REQ-L4-10 |

### 3.2 `Memoria` (64 bits)

| Bits | Campo | Tipo | Domínio | Lei |
| --- | --- | --- | --- | --- |
| 0..=13 | `clausurae` | u14 | anel; bit 0 = ciclo presente | REQ-L4-07, L4-08, L8-01 |
| 14..=15 | `quadrans` | u2 | último quadrante tocado | REQ-L7-01 |
| 16..=19 | `eventus` | u4 | 0..=7 clinamina; 11..=15 reservados | REQ-L4-01 |
| 20..=21 | `ramus` | u2 | 0..=2 ramo; 3 aguardando | REQ-L4-02 |
| 22..=31 | `gradus` | u10 | 0..=1023, saturante | REQ-L4-03, L4-04 |
| 32..=63 | `oraculum` | u32 | 4 linhas × 4 células × 2 bits | REQ-L7-01..03 |

### 3.3 Persistência

Texto de 32 dígitos hexadecimais: `Verbum` (16) seguido de `Memoria` (16), nibble mais
significativo primeiro. Independente de endianness. Único dado gravado por qualquer hospedeiro.

## 4. Contrato de hospedeiro

```rust
pub struct Facultates {
    pub width: u16, pub height: u16,
    pub color_bits: u8,            // 1, 4, 8, 24
    pub max_hz: u8,                // 0 = somente estático
    pub touch: bool, pub motion: bool, pub occlusion: bool, pub persistence: bool,
    pub cells: bool,               // exibe células de glifos nativamente
    pub australis: bool,
    pub tz_offset_min: i16,
    pub lingua: Lingua,            // En, Pt
    pub compute: u8,               // 0..=255
}

pub struct Indicia {
    pub epoch_ms: i64,
    pub energy: u8, pub seism: u8, pub automatism: u8,
    pub idle_s: u32, pub returned_ms: u32,
    pub occluded: bool, pub leaving: bool,
    pub touch: Option<(u8, u8)>,
    pub offset: u8, pub tilt: i8,
}

pub struct Regimen {
    pub stilus: Stilus,            // Creta, Musivum, Littera, Bicolor
    pub strata: u8,                // 1..=4
    pub tick_ms: u32,              // cadência do mundo: 42, 125, 250, 60_000
    pub interval_ms: u32,          // cadência de apresentação, >= tick_ms
    pub layers: u8,                // CAELUM 1, MARGO 2, ANNALES 4, PERSONAE 8, RUINAE 16, IMPOSSIBILIA 32, HEXAGRAMMA 64, PUNCTA 128
    pub lines: u8,                 // 0..=6
}

pub struct Tabula<'a> { pub width: u16, pub height: u16, pub pixels: &'a mut [u8] }
pub struct Cellae<'a> { pub cols: u16, pub rows: u16, pub cells: &'a mut [u16] }

pub struct Nuntius(pub u8);        // CLAUSURA 1, CLINAMEN 2, PERSONA 4, AGNITIO 8, METALEPSIS 16, RARUM 32, LAPSUS 64
```

Protocolo por sessão:

1. O hospedeiro mede o meio e constrói `Facultates` (dimensões, cor, cadência, entradas, língua,
   hemisfério, fuso, orçamento). Versões de sistema e APIs disponíveis entram em `Facultates`
   como capacidades, nunca como nomes.
2. Lê o estado persistido (32 dígitos) ou chama `genesis(seed)` com semente derivada do relógio.
3. Chama `consilium(&facultates, &mundus, energy)` e guarda o `Regimen`.
4. Hospedeiro vivo: a cada `tick_ms` constrói `Indicia` e chama `step`; a cada `interval_ms`
   chama `render` (ou `render_cellae`), mapeia a luminância pela paleta e exibe. Hospedeiro
   estático: a cada execução chama `advance` com os ticks decorridos (REQ-H-06) e depois
   `render`. Reavalia `consilium` quando a faixa de energia, as facultates ou o `modus` mudam.
5. Em oclusão, chama `step` uma única vez com `occluded = true` e suspende a cadência.
6. Antes de ocultar a obra por retorno do observador, chama `step` e `render` com
   `leaving = true` uma vez e exibe esse quadro durante o esmaecimento.
7. Persiste o estado em marcos (`Nuntius` com CLAUSURA, CLINAMEN ou AGNITIO) e ao encerrar.

## 5. Formatos de saída

Pixel `u8`: bits 0..=1 estrato; 2..=3 rótulo de aresta; 4..=7 luminância. Paleta: 16 entradas
RGB por `palette(&concentus)`. Célula `u16`: bits 0..=7 glifo; 8..=11 tom; 12..=13
estrato. Índice de glifo: 0..=69, a posição em `ORDO`; 70..=81, as maiúsculas acentuadas
ÁÀÂÃÉÊÍÓÔÕÚÇ nesta ordem (`ACUTAE`); 57 (`·`) para qualquer outro caractere.

Ordem de composição em `render` (todas as coordenadas passam pela projeção de curvatura):

0. `concentus(&mundus, &facultates, &indicia, &regimen)`: calcula o vetor de afinação consumido
   por todas as etapas seguintes; `palette(&concentus)` deriva as 16 cores; nenhuma etapa lê o estado para decidir cor, contraste, espessura ou registro;
1. ardósia (luminância 1, estrato 3);
2. céu: constelação, grafo das crônicas, hexagrama, lua, linha do evento raro (estrato 2);
3. estratos da encosta (perfil pela regra 30 evoluída por coluna), ruínas (estratos 2..=1);
4. moldura orgânica (estrato 0);
5. figura, rocha, sombra, persona, ensō (estrato 1);
6. crônicas e, após agnitio, o `Verbum` em hexadecimal (estrato 0);
7. Bicolor: limiar de Bayer sobre a luminância; Littera: mapa para a rampa de glifos.

## 6. ABI C (`sisyphium-limen`)

```c
uint32_t sisyphium_version(void);                       /* maior<<16 | menor<<8 | correção */
void     sisyphium_genesis(uint64_t seed, uint64_t *verbum, uint64_t *memoria);
int32_t  sisyphium_parse(const uint8_t text[32], uint64_t *verbum, uint64_t *memoria); /* 1 válido */
void     sisyphium_format(uint64_t verbum, uint64_t memoria, uint8_t out[32]);
void     sisyphium_consilium(const SyFacultates *f, uint64_t verbum, uint64_t memoria, uint8_t energy,
                              SyRegimen *out);
uint32_t sisyphium_step(uint64_t *verbum, uint64_t *memoria, const SyIndicia *i, const SyRegimen *r); /* Nuntius */
uint32_t sisyphium_advance(uint64_t *verbum, uint64_t *memoria, const SyIndicia *i, uint32_t ticks,
                            uint32_t tick_ms, const SyRegimen *r);                                   /* Nuntius */
void     sisyphium_render(uint64_t verbum, uint64_t memoria, const SyFacultates *f, const SyRegimen *r,
                            const SyIndicia *i, uint8_t *pixels, uint16_t width, uint16_t height);
void     sisyphium_palette(uint64_t verbum, uint64_t memoria, const SyFacultates *f, const SyRegimen *r,
                            const SyIndicia *i, uint8_t out[48]);
```

`SyFacultates`, `SyIndicia` e `SyRegimen` são `#[repr(C)]` com os mesmos campos de §4;
`touch` vira `touch_x`, `touch_y` e `has_touch`. A exportação wasm usa as mesmas funções com um
buffer de pixels estático dimensionado pelo hospedeiro.

## 7. Determinismo e totalidade

- Sem relógio, sem aleatoriedade e sem E/S no núcleo: tempo e meio entram por `Indicia`.
- Toda aritmética do núcleo é saturante ou modular por escolha explícita; lints proíbem
  `+`, `-`, `*` implícitos em tipos primitivos (`arithmetic_side_effects`).
- Todo acesso a fatia usa `get`/`get_mut` ou iteradores; lints proíbem indexação direta.
- Os testes de propriedade enumeram sementes pelo próprio gerador de Galois (65 535 casos) e
  combinam com 8 vetores de `Indicia` fixos.

## 8. Hospedeiros

| Hospedeiro | Mede | Exibe | Persistência |
| --- | --- | --- | --- |
| `tabula` | argumentos: `--epoch`, `--energy`, `--seism`, `--lingua`, `--width`, `--height`, `--stilus`, `--state`; recupera o tempo decorrido com `advance` | arquivo PPM ou PNG | arquivo de 32 dígitos e época da última execução |
| `textus` | tamanho do terminal; relógio; energia por argumento ou leitura do sistema quando disponível | sequências ANSI; Littera nativo ou meio-blocos `▀▄` em 16 tons | arquivo de 32 dígitos |
| `limen` | nada | nada | nada; o chamador persiste |
| `hosts/macos` | `pmset` (energia), `sw_vers` (versão), tamanho da tela | `osascript` define a imagem da mesa; `launchd` agenda | arquivo em `~/Library/Application Support/sisyphium/` |
| `hosts/linux` | `/sys/class/power_supply`, `xrandr` ou `swaymsg` | `feh` ou `swww`; `systemd` timer | `$XDG_STATE_HOME/sisyphium/` |
| `hosts/windows` | `Get-CimInstance Win32_Battery`, resolução | `SystemParametersInfo` via PowerShell; agendador de tarefas | `%LOCALAPPDATA%\sisyphium\` |
| `hosts/tela` | tamanho do canvas; `navigator.getBattery` quando existe; visibilidade da página; toque e ponteiro | canvas 2D por `putImageData` | `localStorage` com 32 dígitos |

A sondagem de versão e de API em cada hospedeiro produz apenas `Facultates`; o núcleo não conhece
nomes de sistemas.
