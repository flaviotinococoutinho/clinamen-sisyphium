# 04 — Arquitetura

## 1. Workspace

```
clinamen-sisyphium/
  Cargo.toml                 workspace
  crates/
    sisyphium/               núcleo: no_std, sem alloc, sem dependências, sem unsafe, sem ponto flutuante
    sisyphium-hospes/         biblioteca std: argumentos, tempo, seleção e persistência
    sisyphium-tabula/         hospedeiro: linha de comando; escreve PPM e PNG (codificador próprio)
    sisyphium-textus/         hospedeiro: terminal; Littera nativo; raster por meio-bloco
    sisyphium-limen/          hospedeiro: ABI C (staticlib, cdylib); exportação wasm
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
| `consilium` | L1 | faixas de energia; custo, valor e orçamento; busca satisfatória por recozimento para escolher `Regimen` | `hospes`, `mundus`, `verbum`, `memoria`, `natura`, `spatium` |
| `spatium` | L2 | Q16.16; seno de Bhaskara; isqrt; CORDIC; dispersão; projeção de curvatura; estratos; dominância; dimétrica; escada de Penrose; composição | — |
| `corpus` | L3 | marcha; postura; figuras de blocos; rocha como leitura do `Verbum`; lasca | `verbum`, `spatium`, `signum::raster`, `signum::pixel` |
| `arbitrium` | L4 | clinamina; annales (corpus en/pt); passo; leis de onus, conatus, tranquillitas, ascensus; personae; modus; agnitio; curvatura; contemplação; fecho; interrupção | `verbum`, `memoria`, `natura`, `hospes` |
| `concentus` | L5 | `Concentus`: vetor de afinação de todos os canais (tom, contraste, espessura, cadência, registro do texto, curvatura, índice sonoro) derivado do estado, da efeméride e do `Regimen` de `hospes` | `verbum`, `ephemeris`, `hospes`, `mundus`, `arbitrium` |
| `signum` | L5 | pixel; paleta; stili; fonte; primitivas (linha, disco, retângulo, texto); chicote; grafo do céu; hexagrama; lua; ruínas; máscaras de espaço negativo; compositor de cena | `spatium`, `corpus`, `arbitrium`, `concentus` |
| `sensus` | L6 | mutação em oclusão; anomalia liminar; porta de evento raro; inibição de retorno | `verbum`, `natura` |
| `transumptio` | L7 | oráculo; armadilha; metalepse; amostragem em abismo | `verbum`, `memoria`, `arbitrium`, `spatium`, `hospes` |
| `hospes` | limiar | `Facultates`, `Indicia`, `Tabula`, `Cellae`, `Lingua`, `Nuntius` | — |
| `mundus` | — | `Mundus`: genesis, step, advance, render, render_cellae, format, parse | todos |

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
| 40..=47 | `curvatura` | i8 | −64..=64 depois de todo tick; qualquer valor lido é válido | REQ-L4-11 |
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
significativo primeiro, sem quebra de linha. Independente de endianness. Esse texto é o único
conteúdo persistido do mundo. Imagens, agendas, locks vazios e temporários exclusivos são
artefatos operacionais, sem histórico de interação ou relógio auxiliar (REQ-D-02).

A persistência dos executáveis é opcional, selecionada por `--state`. O diretório pai DEVE
existir. `Archivum` canonicaliza o pai e recusa symlink no componente final, inclusive
symlink pendente. A proteção pressupõe diretório privado e escritores cooperantes; não
protege contra substituição hostil de componentes, hardlinks ou escrita externa ao protocolo.

A leitura e a gravação adquirem o lock vazio `<state>.lock` por criação exclusiva. O lock
permanece somente durante a transação. Um lock preexistente causa erro e NÃO DEVE ser removido
automaticamente. A recuperação de lock órfão exige verificar a ausência de escritor ativo.
Somente arquivo ausente permite genesis. Estado ilegível, com tamanho diferente de 32 bytes
ou com dígito inválido causa erro sem substituição. `--seed` com estado existente causa erro.

A fotografia volátil contém os bytes e o `mtime` opcional obtidos pelo mesmo descritor. Ao
salvar, `Archivum` relê a fotografia sob lock e compara presença, bytes e `mtime`; divergência
causa erro sem repetição automática. Bytes iguais aos solicitados preservam o arquivo e seu
`mtime` depois dessa comparação. Dígitos maiúsculos válidos são normalizados na gravação.

Uma gravação cria temporário exclusivo no mesmo pai, escreve os 32 bytes, solicita `sync_all`,
fecha o descritor e substitui o nome por `rename`, sem apagar previamente o destino. A
fotografia é atualizada após `rename`. Em Unix, a rotina solicita também sincronização do
pai. A operação remove apenas lock e temporário próprios e informa falhas de limpeza.

Uma falha após `rename` ou na liberação do lock pode coexistir com o novo estado publicado.
O diagnóstico informa essa possibilidade; a instância recusa novas gravações após publicação
incerta. O executável DEVE encerrar sem repetição ou restauração automática. Em Unix local,
a substituição do nome é atômica; isso não implica durabilidade absoluta após apagão. Não há
garantia equivalente para todos os sistemas de arquivos ou para arquivos remotos. Bytes e
`mtime` admitem uma sequência de alterações que retorna à mesma fotografia; sem `mtime`, a
comparação protege apenas os bytes sob o lock cooperante.

`write_atomic` aplica escrita, sincronização e substituição a imagens sem impor o formato de
32 bytes. A rotina também recusa symlink no destino e nunca apaga o arquivo anterior antes
de `rename`. Ela não oferece o protocolo de comparação de fotografias de `Archivum`.

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
2. Lê o estado persistido (32 dígitos) ou, quando ausente, chama `genesis(seed)` com semente
   explícita ou derivada dos bytes do relógio. Estado inválido causa erro, sem genesis.
3. Chama `consilium(&facultates, &mundus, energy)` e guarda o `Regimen`.
4. Hospedeiro vivo: a cada `tick_ms` constrói `Indicia` e chama `step(&facultates, &indicia)`;
   a cada `interval_ms` chama `render` (ou `render_cellae`), mapeia a luminância pela paleta e
   exibe. Hospedeiro estático: a cada execução chama
   `advance(&facultates, &indicia, ticks, tick_ms)` com os ticks decorridos (REQ-H-06) e depois
   `render`. A cada medida e após a evolução do mundo, compara as facultates, a faixa de
   energia, o `modus`, o orçamento calculado e a chave semântica de REQ-C-05; reavalia
   `consilium` quando algum deles muda. Essa comparação não executa a busca incondicionalmente
   em todo tick. Em `tabula`, o tempo decorrido vem do `mtime` do próprio arquivo de estado,
   conforme REQ-H-06; uma época sintética exige contagem explícita de ticks. O avanço estático
   usa o `tick_ms` do regimen anterior e desenha com o regimen reavaliado. Em execução contínua,
   o próximo prazo usa o novo período.
5. Em oclusão, chama `step(&facultates, &indicia)` uma única vez com `occluded = true` e
   suspende a cadência.
6. Antes de ocultar a obra por retorno do observador, chama `step` e `render` com
   `leaving = true` uma vez e exibe esse quadro durante o esmaecimento.
7. Persiste o estado em marcos (`Nuntius` com CLAUSURA, CLINAMEN ou AGNITIO) e ao encerrar.

### 4.1 Biblioteca comum

`sisyphium-hospes` depende apenas do núcleo e usa `std`. `argumenta` valida opções únicas e
constrói `Communia`; `tempus` converte o relógio e limita o avanço; `electio` conserva o
regimen até uma transição de REQ-C-05; `archivum` implementa §3.3. `tabula` e `textus` usam
essas leis. `limen` depende diretamente do núcleo. Nenhum cache integra o estado persistido.

`Electio::select` compara todas as `Facultates`, a faixa de energia, `Modus`, o orçamento
calculado e os seis predicados de REQ-C-05. Uma mudança de energia dentro da mesma faixa pode
alterar o orçamento. Uma mudança apenas de `semen` não invalida o regimen. O hospedeiro DEVE
consultar a seleção antes e após a evolução; uma chave igual não executa nova busca.

`tempus::epoch_ms` retorna ausência se o relógio não cabe em `i64` de milissegundos.
`elapsed_ticks` calcula a diferença em `i128`, usa zero para `mtime` ausente ou futuro e
limita a divisão inteira a 65 536 ticks. Período zero produz zero ticks. O excesso além desse
limite é descartado; nenhum resto temporal é gravado. O tempo monotônico governa a espera
do hospedeiro contínuo; o relógio civil alimenta somente `Indicia` e o avanço inicial.

Antes de alocar, dimensões DEVEM ser positivas e seu produto DEVE ser verificado.
`MAX_PIXELS` limita o raster a 33 554 432 pixels, `MAX_CELLS` limita a grade a 1 048 576
células e `MAX_TERMINAL_BYTES` limita um quadro de terminal codificado a 67 108 864 bytes.
Esses limites operacionais são independentes e não alteram os domínios do núcleo.

### 4.2 Opções dos executáveis

Opções desconhecidas, repetidas, sem valor exigido ou fora do domínio causam erro. Valores
numéricos exigem texto UTF-8; caminhos preservam a representação nativa. O parser retorna
`io::Result`. O executável converte falhas de parsing em `Error::Usus` (saída 2) e falhas
operacionais em `Error::Operatio` (saída 1), pela origem e não por `ErrorKind`. Sucesso,
`--help` e `--version` saem com 0; ajuda e versão não produzem efeitos em disco.

| Opção comum | Domínio | Padrão |
| --- | --- | --- |
| `--epoch` | `i64`, milissegundos UTC; exige `--ticks` | relógio do sistema |
| `--ticks` | 0..=65 536; precede `mtime` | tempo decorrido conservador |
| `--energy` | 0..=255 | 255 |
| `--seism` | 0..=255 | 0 |
| `--lingua` | `en`, `pt` | `en` |
| `--seed` | `u64` decimal; incompatível com estado existente | bytes do relógio convertidos a `u64` |
| `--state` | caminho nativo | sem persistência |
| `--compute` | 0..=255 | 255 |
| `--color-bits` | 1, 4, 8, 24 | 8 |
| `--tz-offset` | −1 439..=1 439 minutos | −180 |
| `--australis` | `true`, `false` | `true` |

`tabula` recebe `--width` e `--height` em 1..=65 535, padrão 320×180, respeitando
`MAX_PIXELS`; `--output` padrão `-`; `--format png|ppm` padrão `png`, sem inferência por
extensão. A apresentação estática declara `max_hz = 0`; `--max-hz` não se aplica.
`--stilus creta|musivum|littera|bicolor` exige `--metire`; a medição recusa `--state`,
`--output` e `--ticks` e não publica imagem. Sem stilus explícito, mede as quatro famílias.

`textus` recebe `--cols` e `--rows` em 1..=65 535, `--cells true|false` padrão `true`,
`--frames` em 1..=`u32::MAX`, `--max-hz` em 0..=24 padrão 24 e `--plain`. Sem `--frames`,
a execução é contínua somente em terminal compatível; saída redirecionada ou plain usa um
quadro. `--epoch` exige `--ticks` e um quadro. `--plain` proíbe sequências ANSI. Sem suporte
ANSI comprovado, a saída usa plain. `--max-hz 0` permite no máximo um quadro por minuto,
com espera; não implica repetição ocupada. `textus` não recebe opções de stilus ou medição.

`textus` salva marcos e encerramento controlado, sem escrita a cada quadro. Os sinais
admitidos em [ADR 0008](adr/0008-fronteiras-nativas-dos-hospedeiros.md) somente solicitam o
encerramento; a gravação ocorre fora do callback. Encerramento abrupto, apagão ou fechamento
de janela no sistema que não admite esse sinal podem perder a evolução desde o último marco.

## 5. Formatos de saída

Pixel `u8`: bits 0..=1 estrato; 2..=3 rótulo de aresta; 4..=7 luminância. Paleta: 16 entradas
RGB por `palette(&concentus)`. Célula `u16`: bits 0..=7 glifo; 8..=11 tom; 12..=13
estrato. Índice de glifo: 0..=69, a posição em `ORDO`; 70..=81, as maiúsculas acentuadas
ÁÀÂÃÉÊÍÓÔÕÚÇ nesta ordem (`ACUTAE`); 57 (`·`) para qualquer outro caractere.

Ordem de composição em `render` (as crônicas do plinto usam coordenadas planas; os demais
elementos passam pela projeção de curvatura):

0. `concentus(&mundus, &facultates, &indicia, &regimen)`: calcula o vetor de afinação consumido
   por todas as etapas seguintes; `palette(&concentus)` deriva as 16 cores; nenhuma etapa lê o estado para decidir cor, contraste, espessura ou registro;
1. ardósia (luminância 1, estrato 3);
2. céu na região `sky_region(comp, W, onus)`: constelação, grafo das crônicas, hexagrama,
   lua (estrato 2);
3. estratos da encosta (perfil pela regra 30 evoluída por coluna), ruínas (estratos 2..=1):
   cada traço tem tom ≤ 15−`contrastus` e recorte projetado em y < S; a projeção mantém o
   centro e as dimensões do quadro completo;
4. moldura orgânica (estrato 0);
5. sombra, figura e rocha (estrato 1): a sombra aponta para a direita na janela liminar;
   a postura usa `contemplating` e `asleep`; a rocha recebe a microtextura em contemplação
   quando a escala da célula é ≥ 8 e a costura kintsugi em AmorFati. O raio raro liga o ponto
   de Hénon no céu ao topo da cabeça (tom 15, estrato 2). Personae e atributos entram com
   PERSONAE: Diógenes, lâmpada; Heráclito, pira; daemon no cume. O ensō aparece no quadrante
   previsto com armadilha pronta, exceto em Somnium. Impossibilia usa a escada fechada de
   span 6 e escala 2s, sob IMPOSSIBILIA ∧ ((`conatus` > 200 ∧ `onus` > 200) ∨ Somnium),
   com classes de dominância e rótulos por lance. Todos esses elementos precedem o
   aforismo do nó 0 com fundo
   ardósia de (6·glifos+2)·s por 12s em (x−s,y−s), sob a projeção, estrato 2 e recorte
   projetado em y < S. A âncora nominal é (nó0.x+2s,nó0.y−5s). A caixa
   do hexagrama ativo (incluindo espessura) e a caixa do disco lunar noturno iluminado,
   expandidas em 1s como respiro lógico, são protegidas. Entre y nominal e as
   posições imediatamente acima/abaixo de cada caixa, vence o menor deslocamento que caiba
   em 0..S, mantenha a caixa projetada dentro do quadro acima do plinto e evite ambas nos
   espaços lógico e projetado, desempate pelo menor y. Os limites projetados reutilizam
   a projeção e a cobertura retangular de `plot`, incluindo o vizinho após a última coluna
   e a última linha lógicas; largura e altura permanecem independentes. Sem candidato com x nominal, tenta distâncias horizontais inteiras
   crescentes, esquerda antes de direita em empate, limitadas pela largura do quadro, com os
   mesmos candidatos verticais. Vence a primeira posição viável; a caixa também precisa caber
   logicamente na largura. Esgotada essa busca, repete a ordem horizontal com até seis
   candidatos nas fronteiras projetadas do quadro e dos dois signos (topo/base e acima/abaixo).
   A busca binária usa limites verticais não decrescentes na altura lógica, dentro do quadro
   e em κ operacional ±64; o custo adicional é O(W·log S), sem varredura de área. Cada
   candidato passa pela validação completa de contenção e disjunção. Sem posição viável,
   omite o aforismo nesse quadro;
6. crônicas com `Pictor::flat`, fora da projeção; após agnitio, o `Verbum` presente em
   hexadecimal sob a projeção da cena (estrato 0);
7. Bicolor: limiar de Bayer sobre a luminância; Littera: mapa para a rampa de glifos.

`Mundus::render` obtém o prefixo efetivo de `arbitrium::visible_annales`: ANNALES,
`regimen.lines` e o limite do meio governam juntos o texto e seu registro. O caminho raster
calcula o acorde com `cells = false`, preservando as demais capacidades e o regimen escolhido.
`render_cellae` normaliza as capacidades para `cells = true`, `width = cols`, `height = rows` da grade real
antes de calcular `Concentus`; a seleção usa o mesmo limite de linhas e de `rows`. A
identidade do acorde entre stili pressupõe capacidades iguais. Os dígitos de agnitio são
independentes da camada ANNALES. O aforismo pertence a CAELUM e usa a seleção completa de
`annales`, mesmo quando a capacidade efetiva de crônicas é zero.

## 6. ABI C (`sisyphium-limen`)

```c
uint32_t sisyphium_version(void);                       /* maior<<16 | menor<<8 | correção */
void     sisyphium_genesis(uint64_t seed, uint64_t *verbum, uint64_t *memoria);
int32_t  sisyphium_parse(const uint8_t text[32], uint64_t *verbum, uint64_t *memoria); /* 1 válido */
void     sisyphium_format(uint64_t verbum, uint64_t memoria, uint8_t out[32]);
void     sisyphium_consilium(const SyFacultates *f, uint64_t verbum, uint64_t memoria, uint8_t energy,
                              SyRegimen *out);
uint32_t sisyphium_step(uint64_t *verbum, uint64_t *memoria, const SyFacultates *f, const SyIndicia *i); /* Nuntius */
uint32_t sisyphium_advance(uint64_t *verbum, uint64_t *memoria, const SyFacultates *f, const SyIndicia *i,
                            uint32_t ticks, uint32_t tick_ms);                                           /* Nuntius */
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

### 7.1 Ordem do tick

`step(&facultates, &indicia)` aplica as leis na ordem abaixo, com
`eph` = `ephemeris(epoch_ms, tz_offset_min, australis)`. `mundus` compõe as leis dos módulos
nessa ordem. A composição também aplica a guarda do fecho no cume, retém o passo em
contemplação e incrementa `ascensus` quando a porta do passo abre (REQ-L4-05, REQ-L4-07).

| # | Etapa | Condição | Anúncio | Requisito |
| --- | --- | --- | --- | --- |
| 1 | gerador: `semen` ← `galois(semen)` | sempre | — | REQ-L1-01 |
| 2 | fecho do ciclo | `ascensus` = 255 | CLAUSURA | REQ-L4-07 |
| 3 | interrupção | `leaving` | — | REQ-L4-08 |
| 4 | oclusão | `occluded` | — | REQ-L6-01 |
| 5 | lasca | `seism` ≥ 240 ∧ (`semen` ∧ 0x3F) = 0 | LAPSUS | REQ-L3-04 |
| 6 | toque | `touch` presente | METALEPSIS | REQ-L7-01, REQ-L7-03 |
| 7 | modus | sempre | AGNITIO | REQ-L4-10, REQ-L4-11 |
| 8 | passo: `ascensus` + 1 | fora da contemplação, com a porta do passo aberta | — | REQ-L4-05 |
| 9 | leis contínuas | sempre | — | REQ-L4-06 |
| 10 | evento: idade, fim, clinamen, persona | sempre | CLINAMEN ou PERSONA | REQ-L4-02, REQ-L4-03, REQ-L4-09 |
| 11 | dinâmica da curvatura | sempre | — | REQ-L4-11 |
| 12 | porta rara | janela ativa | RARUM | REQ-L6-03 |

O anúncio de um tick é a união dos anúncios das suas etapas; o de `advance` é a união dos
anúncios dos seus ticks. As razões da ordem:

- O gerador abre o tick: todas as portas das etapas seguintes leem o `semen` já avançado.
- O fecho precede o passo: o cume alcançado por um passo fica à vista até o fim do tick e fecha no
  início do tick seguinte.
- A interrupção segue o fecho e marca o ciclo que o fecho abre.
- A oclusão e a lasca precedem o passo, que lê o vento no `automatum` já mutado (REQ-L1-08).
- O toque precede o modus: a lei do toque lê o modus antes da lei do modus, e por isso em Somnium
  não há metalepse (REQ-L7-03); a lei do modus recebe o toque como estímulo e acorda o sonho
  (REQ-L4-10). A lei do modus tem a última palavra: com `energy` < 39 ela põe Quies mesmo no tick
  de uma metalepse, que mantém o evento e o anúncio.
- O modus precede o passo: a contemplação (Quies; sono após a agnitio) retém o passo no mesmo
  tick.
- O passo precede as leis contínuas: o `conatus` se gasta no tick do passo (REQ-L4-06).
- A curvatura segue o evento: a visita de Heráclito soma +8 à `curvatura` dentro do evento.
  A dinâmica final reescala a soma em Vigilia e Quies por 3/4 em direção a 0, truncado, e
  limita o resultado a −64..=64 (REQ-L4-11). Assim, uma visita em Vigilia com curvatura 0
  deixa +6 ao fim do tick; uma curvatura inicial em +57..=+64 não sai do tick acima de +64.
- A porta rara fecha o tick: lê a `clausurae` e o `eventus` que o tick deixa.

`step` lê `Facultates` somente para a efeméride (`tz_offset_min`, `australis`) e não lê
`Regimen`. De `Indicia`, não lê `automatism`, `returned_ms`, `offset` nem `tilt`. O toque de
`Indicia` conta mesmo com `Facultates.touch` falso: o toque é medida.

## 8. Hospedeiros

| Hospedeiro | Mede | Exibe | Persistência |
| --- | --- | --- | --- |
| `tabula` | argumentos: `--epoch`, `--ticks`, `--energy`, `--seism`, `--lingua`, `--width`, `--height`, `--stilus`, `--state`; mede o tempo decorrido pelo `mtime` do próprio arquivo de estado e chama `advance` (REQ-H-06); `--epoch` sintético exige `--ticks` | arquivo PPM ou PNG | arquivo de exatamente 32 dígitos; `mtime` como medida externa |
| `textus` | tamanho do terminal; relógio; energia por argumento ou leitura do sistema quando disponível | sequências ANSI; Littera nativo ou meio-blocos `▀▄` em 16 tons | arquivo de 32 dígitos |
| `limen` | nada | nada | nada; o chamador persiste |
| `hosts/macos` | `pmset` (energia), `sw_vers` (versão), tamanho da tela | `osascript` define a imagem da mesa; `launchd` agenda | arquivo em `~/Library/Application Support/sisyphium/` |
| `hosts/linux` | `/sys/class/power_supply`, `xrandr` ou `swaymsg` | `feh` ou `swww`; `systemd` timer | `$XDG_STATE_HOME/sisyphium/` |
| `hosts/windows` | `Get-CimInstance Win32_Battery`, resolução | `SystemParametersInfo` via PowerShell; agendador de tarefas | `%LOCALAPPDATA%\sisyphium\` |
| `hosts/tela` | tamanho do canvas; `navigator.getBattery` quando existe; visibilidade da página; toque e ponteiro | canvas 2D por `putImageData` | `localStorage` com 32 dígitos |

A sondagem de versão e de API em cada hospedeiro produz apenas `Facultates`; o núcleo não conhece
nomes de sistemas.
