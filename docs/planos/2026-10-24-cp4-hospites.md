# Checkpoint 4 — Hospites — plano de implementação

> Execução por tarefas, com revisão independente de especificação e de implementação a cada
> entrega. Método autorizado: execução dirigida por subtarefas. Os passos usam caixas de seleção.

**Estado:** plano versionado para execução. Base documental integrada em main:
`cb647a4109a596d0e45df333906d0ce37b20b8ac` (PR5); ramo `feat/hospites`. Plano em `docs/planos/2026-10-24-cp4-hospites.md`.
**Objetivo:** entregar tabula, textus, limen, scripts dos três sistemas e custos medidos de consilium.
**Arquitetura:** três hospedeiros finos consomem o núcleo de 128 bits. Uma biblioteca interna
compartilha somente leis de CLI, relógio, escolha e persistência entre os dois executáveis.
O ABI converte representações C explicitamente; unsafe permanece apenas no ABI e nos sinais
do terminal. O núcleo mantém forbid e suas demais restrições.
**Tecnologias:** Rust 1.85/edição 2024; std nos hospedeiros; C11 para consumidor ABI;
Python padrão para verificação de artefatos; shell e PowerShell para integração de sistema.
**Especificação:** `docs/01-visao.md` M2–M4 e checkpoint 4; `03-especificacao.md` REQ-C/H/D;
`04-arquitetura.md` §§3–6,8; `06-consilium.md`; `08-verificacao.md`; `convencoes.md`;
ADRs 0003/0004. Resoluções deste plano integram os documentos técnicos na tarefa 1.

## Restrições globais

- Núcleo `no_std`, sem alloc, dependências, unsafe ou floats; `Mundus` ocupa 16 bytes.
- Estado do mundo: exatamente 32 dígitos hexadecimais, sem newline, sem sidecar cronológico.
- Hospedeiros sem rede, histórico de interação, captura de atenção ou dependência de imagem/terminal.
- Nomes novos de módulos/tipos/campos em latim; funções/rustdoc/commits em inglês; docs em português.
- `consilium` escolhe a experiência; cadências, budget e forçamentos permanecem leis do núcleo.
- Sem alteração automática do ambiente: não executar apply/install no computador de trabalho.
- Três plataformas no CI; execução com comandos falsos não prova aplicação nativa de wallpaper.
- Custos só mudam com medição; valor semiótico, orçamento e busca não são recalibrados.
- `docs/canon/` indexa a essência. O plano não transforma parâmetros operacionais em invariantes.
- Nenhuma versão mínima de sistema é inventada; capacidades são sondadas com fallback explícito.

## Focos de revisão

1. Estado corrompido, symlink ou duas invocações concorrentes: nenhum overwrite silencioso (T1).
2. Relógio regressivo, mtime futuro e epoch sintético: avanço conservador e limitado (T1/T2).
3. PNG atravessando fronteira de 65535 bytes e broken pipe: arquivo decodificável ou erro íntegro (T2).
4. Terminal fechado/redimensionado e sinal durante espera: marcos e saída controlada persistidos (T3).
5. ABI com enum inválido, null ou sentinelas adjacentes: erro sem escrita fora do contrato (T4).

## Mapa de arquivos e sequência

| Unidade | Arquivos de produção | Responsabilidade |
| --- | --- | --- |
| Fundação | `crates/sisyphium-hospes/{Cargo.toml,src/lib.rs,src/argumenta.rs,src/tempus.rs,src/electio.rs,src/archivum.rs}` | argumentos comuns, tempo, cache, transação32 |
| Tabula | `crates/sisyphium-tabula/{Cargo.toml,src/main.rs,src/lib.rs,src/imago.rs,src/metire.rs}` | CLI estática, PPM/PNG, medição |
| Textus | `crates/sisyphium-textus/{Cargo.toml,src/main.rs,src/lib.rs,src/terminalis.rs,src/signa.rs}` | sondagem, células/meio-blocos, loop |
| Limen | `crates/sisyphium-limen/{Cargo.toml,src/lib.rs,src/forma.rs,include/sisyphium.h}` | ABI e conversões validadas |
| Integração | `hosts/{macos,linux}/sisyphium.sh`, `hosts/windows/sisyphium.ps1` e templates adjacentes | sondagem, imagem, aplicação e agenda |
| Evidência | `scripts/{verify-hosts.py,verify-ffi.py,verify-images.py,calibrate-costs.py}`, testes nos crates | verificações independentes |

T1 → T2 → gate da fundação → T3; T4 depende somente do núcleo e pode executar em paralelo a T1/T2. T3/T4 → T5 → T6 → T7.
T3 possui apenas textus; T4 possui apenas limen e verify-ffi.py. O controlador integra Cargo.lock
uma única vez após cada rodada; nenhum executor reverte alterações de outra tarefa.
Todos os caminhos abaixo são relativos à raiz. Manifestos herdam package/lints, salvo limen e textus.
Cada tarefa termina com testes, revisão de requisitos, revisão técnica e commit coerente.

## Contratos operacionais decididos

**CLI comum.** `--epoch <i64-ms>` exige `--ticks <0..65536>`; ticks explícitos têm precedência
sobre mtime. Sem epoch, relógio do sistema. `--energy <u8>` padrão255; `--seism <u8>` padrão0;
`--lingua en|pt` padrãoen; `--seed <u64-decimal>` padrão época efetiva convertida a u64 por bytes (época sintética conserva repetibilidade);
`--state <path>` sem padrão (não persiste quando omitido). Seed com estado já existente falha.
`--compute <u8>` padrão255; `--color-bits 1|4|8|24` padrão8; `--tz-offset <-1439..1439>`
padrão−180; `--australis true|false` padrãotrue. Flags repetidas/desconhecidas falham.
Erro de uso=2, operacional=1, sucesso=0; `--help`/`--version` não têm efeitos em disco. O parser retorna io::Error, mas os executáveis preservam sua origem: Error::Usus para falhas de argumentos e Error::Operatio para E/S, mesmo quando ambas têm ErrorKind::InvalidInput.

**Tabula.** Width/height padrão320/180, cada dimensão1..65535 e área≤33_554_432 pixels;
`--output <path|->` padrão`-`; `--format png|ppm` padrãopng, sem inferência pela extensão.
`--max-hz` não se aplica: apresentação estática usa max_hz0; demais entradas não medidas são
falsas/desconhecidas. `--stilus creta|musivum|littera|bicolor` só com `--metire`; esse modo
recusa state/output/ticks e não publica imagem. `--metire` sem stilus mede todas as famílias.

**Textus.** Mesmas flags comuns; `--cols`, `--rows` opcionais em1..65535, área raster também
≤33_554_432. `--cells true|false` padrãotrue; false declara saída raster por meio-blocos.
`--frames <u32>` deve ser≥1; omitido significa contínuo somente em TTY compatível, 1 em pipe/plain.
`--plain` proíbe toda sequência ANSI e usa uma grade por quadro, separada por newline.
`--max-hz <0..24>` padrão24; zero apresenta no máximo um quadro por minuto, sem busy loop.
Epoch sintético exige ticks e frames1 para impedir relógio misto em animação.
Não recebe `--stilus` nem opções de medição. Sem suporte ANSI comprovado, usa plain.

**Estado e interrupção.** Diretório pai existente; não criar árvore implícita no binário.
Rejeitar symlink no caminho do estado; canonicalizar pai. Leitura ausente permite genesis;
leitura inválida, tamanho≠32 ou erro de acesso falha sem escrever. Lock vazio `<state>.lock`,
`create_new`, mantido apenas ao ler/salvar; snapshot inclui bytes e mtime opcionais. Ao salvar,
reler sob lock e comparar snapshot; conflito falha sem retry. Temporário exclusivo no mesmo
pai recebe32, sync_all e rename; atualizar snapshot depois do rename. Unix sincroniza pai.
Remover somente lock/temporário pertencentes à operação; lock preexistente nunca é removido
automaticamente. Falha abrupta pode deixar esses artefatos e requer recuperação manual.
Arquivo antigo nunca é apagado antes de rename; sem promessa de durabilidade absoluta em apagão.
Após rename, erro de sync/cleanup pode coexistir com estado novo publicado: encerrar sem retry
ou rollback. Bytes+mtime admite ABA; sem mtime a proteção é apenas por bytes e lock cooperante.
Textus persiste marcos e encerramento controlado; não grava cada quadro. SIGKILL, apagão e
fechamento de janela Windows não garantem salvamento desde o último marco. Tabula salva por execução.

**Seleção.** Chave de cache contém Facultates, faixa de energia, Modus, budget e os seis
predicados REQ-C-05. Comparar antes e após evolução. No caminho estático, advance usa tick_ms
do regimen prévio e desenha com o regimen reavaliado. No loop, próximo deadline usa novo tick.
A ausência de transição não chama busca. O cache e contadores de teste nunca são persistidos.

### Tarefa 1: fundação compartilhada e contratos documentados

**Arquivos:** criar toda unidade Fundação da tabela e
`crates/sisyphium-hospes/tests/{tempus,electio,archivum,argumenta}.rs`;
alterar `docs/{03-especificacao,04-arquitetura,08-verificacao}.md`; criar
`docs/adr/0008-fronteiras-nativas-dos-hospedeiros.md`; `Cargo.lock` pelo controlador.
O ADR delimita exceções unsafe dos hosts: workspace forbid inalterado; só limen/textus não
herdam lints, copiam todos os demais, usam unsafe_code=deny e allow localizado em fronteiras;
unsafe_op_in_unsafe_fn, undocumented_unsafe_blocks e missing_safety_doc são deny.
Em docs04 registrar o crate interno e contratos acima; REQ-D-02 distingue conteúdo do mundo
(32 bytes) de imagens, agendas, lock vazio e temporário operacional sem histórico.

**Interfaces produzidas:**
- `MAX_PIXELS:usize=33_554_432`, `MAX_CELLS:usize=1_048_576` e `MAX_TERMINAL_BYTES:usize=67_108_864`; `validate_area(width:u16,height:u16,limit:usize)->io::Result<usize>` verifica dimensões positivas e produto checked antes de reservar.
- `Error::{Usus(io::Error),Operatio(io::Error)}` com Display, std::error::Error e `exit_code(&self)->u8` (2 ou 1). Parsers continuam io::Result; os executáveis convertem explicitamente na fronteira de parsing.
- `tempus::epoch_ms(SystemTime)->Option<i64>` e
  `tempus::elapsed_ticks(now:i64,mtime:Option<i64>,tick_ms:u32)->u32`.
- `argumenta::Argumenta::parse(args:&[OsString],allowed:&[(&str,bool)])->io::Result<Self>`;
  `take(&mut self,name:&str)->Option<OsString>`; `finish(self)->io::Result<()>`;
  `Communia::parse(&mut Argumenta)->io::Result<Self>`;
  bool em allowed indica argumento com valor; flag sem valor produz OsString vazia em take.
  `Communia::facultates(width:u16,height:u16,cells:bool,max_hz:u8)->Facultates`;
  `Communia::indicia(epoch_ms:i64)->Indicia`.
- Campos `Communia`: `epocha:Option<i64>,ictus:Option<u32>,energia:u8,sismus:u8,
  lingua:Lingua,semen:Option<u64>,archivum:Option<PathBuf>,computatio:u8,
  profunditas:u8,meridianus:i16,australis:bool`.
- `electio::Electio::new()->Self` e
  `select(&mut self,f:&Facultates,w:&Mundus,energy:u8)->Regimen`.
- `archivum::Archivum::open(path:&Path,seed:Option<u64>,fallback_seed:u64)
  ->io::Result<(Self,Mundus)>`; `save(&mut self,world:&Mundus)->io::Result<()>`;
  `mtime(&self)->Option<SystemTime>`. Archivum guarda caminho e snapshot volátil.
- `archivum::write_atomic(path:&Path,write:impl FnOnce(&mut File)->io::Result<()>)->io::Result<()>`
  para imagem streaming; não aplica contrato32 e nunca apaga o destino para substituí-lo.

- [ ] Escrever `elapsed_ticks_conservative`: now1000/mtime958/tick42→1;
  future1001→0; None→0; i64::MAX−i64::MIN satura65536; tick0→0.
- [ ] Escrever `argument_domains`: epoch sem ticks,65537,dimensão0,enum inválido,
  duplicado, não UTF8 em valor numérico→erro; path não UTF8 aceito; ticks0 aceito.
- [ ] Escrever `cache_transition_only`: chamada repetida→uma busca; energia dentro da mesma
  faixa com budget distinto, troca de modus, cada predicado semântico e Facultates→nova busca;
  mera troca de semen que não altera chave→sem busca. Instrumentação fica só nos testes.
- [ ] Escrever `state_transaction_preserves_previous`: inválidos0/31/33 bytes e hex ruim
  permanecem idênticos; uppercase normaliza ao salvar; seed conflitante falha; lock preexistente
  falha; dois snapshots concorrentes rejeitam segundo save; symlink é recusado.
- [ ] Escrever `atomic_failure_keeps_committed_state`: injetar falha antes de rename por
  helper privado de teste; snapshot antigo intacto; só temporário próprio removido; save normal
  lê32 bytes sem newline. Injetar falha após rename: novo estado pode estar publicado, erro
  é reportado e não dispara rollback. Nunca matar processos do usuário em teste.
- [ ] Executar `cargo test -p sisyphium-hospes`; observar falhas de contrato ausente.
- [ ] Implementar as interfaces, erros sem panic e alocações limitadas; nenhum generic
  framework. O tipo de erro do parser é `std::io::Error` com InvalidInput.
- [ ] Atualizar os três documentos e executar `cargo test -p sisyphium-hospes`, fmt,
  clippy do crate e `scripts/verba.sh`; resultado esperado: todos passam.
- [ ] Revisar preservação de dados e invariantes; commit `feat(hospes): share host time and state laws`.

### Tarefa 2: tabula, imagem independente e execução estática

**Arquivos:** criar unidade Tabula, `crates/sisyphium-tabula/tests/{cli,imago}.rs`,
`scripts/verify-images.py`; alterar docs04§8 e docs08 com uso e provas. `metire.rs` fica para T6;
não criar módulo vazio. O controlador integra Cargo.lock.

**Interfaces consumidas:** Communia, Archivum, Electio e tempus da T1;
`Mundus::{advance,render,palette}`, `signum::pixel::lum_of`.
**Interfaces produzidas:** `imago::Forma::{Png,Ppm}`;
`imago::write_image(format:Forma,width:u16,height:u16,pixels:&[u8],palette:&[[u8;3];16],
output:&mut dyn Write)->io::Result<()>`; `run(args:&[OsString],output:&mut dyn Write)->Result<(),sisyphium_hospes::Error>`.
Binário traduz erro de parse para2; integração não depende de textos de erro literais.

- [ ] Escrever `ppm_maps_luminance_only`: bytes0x10/0x1f mapeiam à mesma cor de índice1;
  P6 cabeçalho `P6\n2 1\n255\n`, exatamente6 bytes RGB; buffer curto falha.
- [ ] Escrever `png_structure_and_boundaries`:1×1,320×180,257×256 exercitam um/múltiplos
  blocos stored e fronteiras de65535 bytes; chunks em ordem IHDR/PLTE/IDAT/IEND, CRC válido, PLTE48 bytes,
  índices0..15; decompressed length=height*(width+1). Sentinel após raster não é lido.
- [ ] Criar verificador Python padrão: struct/binascii/zlib, parse estrito de todos os chunks,
  CRC, zlib.decompress, Adler, filtro0, limites/terminador; comparar RGB decodificado ao PPM
  de cenário idêntico. Não chamar encoder Rust como decoder de referência.
- [ ] Escrever `cli_static_parity`: seed1/epoch1782054000000/ticks4096/energy255/320×180
  compara estado final com advance público e RGB com render público; state ausente não nasce
  quando flag omitida; stdout contém só imagem; defaults png e320×180 comprovados.
- [ ] Escrever `cli_real_mtime`: criar estado, fixar mtime via Python padrão, executar sem
  epoch com ticks explícito em teste determinístico; testar cálculo por mtime diretamente em
  T1. Em CLI normal verificar elapsed conservador, sem asserção de relógio exato instável.
- [ ] Escrever `cli_invalid_state_and_output`: arquivo corrompido permanece intacto; output
  não gravável falha antes de commit do estado; output/state iguais após canonicalização são
  recusados; output igual ao lock ou symlink também falha; stdout broken pipe→1, sem evolução posterior; limites de área/inteiro→2.
- [ ] Executar `cargo test -p sisyphium-tabula`; observar falhas esperadas.
- [ ] Implementar fluxo: parse→validar destinos→carregar→selecionar→advance→selecionar→render→
  escrever imagem streaming→flush→salvar estado. Arquivo usa temporário+rename; stdout pode
  ter prefixo em falha de I/O. Falha de imagem não salva estado; falha posterior de persistência
  pode deixar imagem pronta e estado anterior/incerto, reportada sem rollback de arquivos alheios.
- [ ] Implementar PNG próprio: assinatura8; IHDR tipo3/depth8/compressão0/filtro0/interlace0;
  PLTE16; linhas filtro0; zlib0x78,0x01; stored LEN/NLEN little-endian e BFINAL correto;
  Adler big-endian; CRC-32 inclui tipo+dados; comprimentos de chunk big-endian. IDATs contíguos
  particionam o mesmo stream zlib; buffer stored≤65535, PPM usa uma linha RGB, sem triplicar
  raster inteiro. Checked_mul/try_reserve antecedem efeitos; MAX_PIXELS=33_554_432 é parâmetro
  operacional dos hosts, cobre7680×4320 e não entra no cânone.
- [ ] Executar testes Rust e `python3 scripts/verify-images.py --binary target/debug/sisyphium-tabula`;
  ambos passam, inclusive zlib independente. Inspecionar PNG/PPM de pelo menos vigilia/quies.
- [ ] Revisar T1+T2 como gate compartilhado; commit `feat(tabula): render portable static images`.

### Tarefa 3: textus e cadência finita verificável

**Ownership:** só unidade Textus, `crates/sisyphium-textus/tests/{cli,terminalis}.rs`;
docs04/08 são atualizados pelo controlador com nota da tarefa, sem edição concorrente.
**Consome:** T1; core render_cellae/render/palette e ORDO/ACUTAE.
**Produz:** `terminalis::measure()->Option<(u16,u16)>`;
`terminalis::format_cells(cells:&[u16],cols:u16,rows:u16,palette:&[[u8;3];16],plain:bool)
->io::Result<Vec<u8>>`;
`terminalis::format_blocks(pixels:&[u8],width:u16,height:u16,palette:&[[u8;3];16],plain:bool)
->io::Result<Vec<u8>>`; `run(args:&[OsString],output:&mut dyn Write)->Result<(),sisyphium_hospes::Error>`;
`signa::Custodia::install()->io::Result<Self>`, `requested()->bool`,
`Custodia::restore(self)->io::Result<()>`. Módulo signa é a única exceção unsafe de textus.

- [ ] Escrever `native_cells_decode`: índices69/70/81 e índice inválido→ORDO/Á/Ç/·;
  tom vem bits8..11, estrato não afeta cor; scratch raster cols×rows.
- [ ] Escrever `blocks_pair_rows`: `▀` foreground=lum superior/background=inferior; última
  linha ímpar usa fundo índice1; plain usa a mesma geometria sem ESC, não RGB textual.
- [ ] Escrever `terminal_output_bound`: reservar pior caso checked dos escapes RGB, UTF8 e separadores; limite de células e 64MiB de bytes são independentes de MAX_PIXELS; erro antes de apresentar ou alterar mundo.
- [ ] Escrever `terminal_size_precedence`: flags individuais prevalecem sobre medida, medida
  sobre env COLUMNS/LINES válidos, fallback80×24; reservar uma linha de saída; rows mínimo1.
  Cells usa cols×rows; raster usa cols×(rows*2), com overflow/área recusados.
- [ ] Escrever `finite_frames_and_pipe`: frames2 termina com dois quadros; pipe sem flags
  emite um quadro plain; epoch/ticks/frames1 determinístico; frame0 falha; nenhuma sequência
  de ocultar cursor, alternate screen, query de entrada ou mudança de modo é emitida.
- [ ] Escrever `monotonic_deadlines`: helper puro recebe elapsed e período; não há sleep
  negativo, catch-up≤65536, atraso não provoca busy loop; apresentação respeita interval_ms,
  evolução respeita tick_ms inclusive max_hz0. Relógio civil regressivo não altera espera.
- [ ] Escrever `milestones_and_exit_persist`: contador confirma zero saves por quadro sem
  marco; CLAUSURA/CLINAMEN/AGNITIO e saída controlada salvam32; falha de saída encerra evolução
  e tenta save em contexto normal. Conflito falha sem overwrite. Sinais reais entram em T7.
- [ ] Escrever `signal_registration`: falha da segunda instalação restaura a primeira;
  restauração falha é reportada; callback só store AtomicBool, evento Windows desconhecido→0;
  instalação sem alvo admitido falha antes de efeitos em execução contínua.
- [ ] Executar `cargo test -p sisyphium-textus`; observar falhas esperadas.
- [ ] Implementar sondagem sem dependência: Unix `stty size` com stdin herdado e erro silencioso;
  Windows PowerShell `[Console]::WindowWidth/WindowHeight`; processos só na inicialização e em apresentações, no máximo uma sondagem por segundo; dimensões explícitas completas dispensam sondagem. Nunca consultar um processo a cada tick ou quadro de 24 Hz. Testar ausência/erro com executável falso.
- [ ] Implementar loop com Instant e thread::sleep até próximo deadline; primeira apresentação
  imediata após catch-up inicial com ticks explícitos ou mtime segundo T2; evolução seguinte
  por tick. Reavaliar cache antes/depois de cada evolução; marcos
  e saída controlada persistem. Redimensionamento realoca com try_reserve e renova
  Facultates; não traduz ausência de foco desconhecida como oclusão inventada.
- [ ] ANSI somente se TTY e capacidade declarada por TERM não dumb (Unix) ou WT_SESSION/TERM
  compatível (Windows); caso contrário plain. Emitir cursor-home somente entre frames ANSI,
  reset de cor por linha e no final. Não alterar modo/cursor visível.
- [ ] Implementar signa usando os contratos primários locais: Linux glibc/macOS64
  x86_64/aarch64, signal(c_int,usize)->usize preserva sentinelas0/1/usize::MAX sem chamar
  retornos; SIGINT2/SIGTERM15. Restaurar disposição anterior, sem alegar restauração de
  máscara/flags sigaction. Windows SetConsoleCtrlHandler usa extern system callback(u32)->i32;
  somente CTRL_C/CTRL_BREAK gravam AtomicBool Relaxed e retornam1; demais retornam0.
  Callback sem I/O/alloc/mutex; target_has_atomic8 obrigatório. Loop consulta flag entre
  sleeps≤250ms, salva e restaura fora do callback. I/O bloqueado impede prazo rígido.
  Nenhuma promessa de save para SIGKILL/TerminateProcess/CLOSE/LOGOFF/SHUTDOWN.
- [ ] Executar testes e smoke `sisyphium-textus --plain --frames 2 --cols 40 --rows 12`;
  capturar saída finita, sem ESC e sem arquivos quando state omitido. Revisar limitações de sinal.
- [ ] Commit `feat(textus): present native cells and half blocks`.

### Tarefa 4: limen, header e consumidor C real

**Ownership:** unidade Limen, `crates/sisyphium-limen/tests/ffi.rs`,
`crates/sisyphium-limen/tests/consumer.c`, `scripts/verify-ffi.py`.
**Consome:** core diretamente. **Produz:** staticlib, cdylib e header C11 das nove funções
exatas de docs04§6; build wasm conserva exports quando alvo disponível, sem página/canvas.

Assinaturas C mantidas (sem símbolo adicional de produto):

```c
uint32_t sisyphium_version(void);
void sisyphium_genesis(uint64_t seed, uint64_t *verbum, uint64_t *memoria);
int32_t sisyphium_parse(const uint8_t text[32], uint64_t *verbum, uint64_t *memoria);
void sisyphium_format(uint64_t verbum, uint64_t memoria, uint8_t out[32]);
void sisyphium_consilium(const SyFacultates *f, uint64_t verbum, uint64_t memoria, uint8_t energy, SyRegimen *out);
uint32_t sisyphium_step(uint64_t *verbum, uint64_t *memoria, const SyFacultates *f, const SyIndicia *i);
uint32_t sisyphium_advance(uint64_t *verbum, uint64_t *memoria, const SyFacultates *f, const SyIndicia *i, uint32_t ticks, uint32_t tick_ms);
void sisyphium_render(uint64_t verbum, uint64_t memoria, const SyFacultates *f, const SyRegimen *r, const SyIndicia *i, uint8_t *pixels, uint16_t width, uint16_t height);
void sisyphium_palette(uint64_t verbum, uint64_t memoria, const SyFacultates *f, const SyRegimen *r, const SyIndicia *i, uint8_t out[48]);
```

Layouts com padding explícito, ordem fixa e inteiros de largura conhecida:

| Estrutura | Campos na ordem (offset em bytes) | Tamanho/alinhamento |
| --- | --- | --- |
| SyFacultates | width:u16@0,height:u16@2,color_bits:u8@4,max_hz:u8@5,touch:u8@6,motion:u8@7,occlusion:u8@8,persistence:u8@9,cells:u8@10,australis:u8@11,tz_offset_min:i16@12,lingua:u8@14,compute:u8@15 |16/2|
| SyIndicia | epoch_ms:i64@0,energy:u8@8,seism:u8@9,automatism:u8@10,reservatum0:u8@11,idle_s:u32@12,returned_ms:u32@16,occluded:u8@20,leaving:u8@21,touch_x:u8@22,touch_y:u8@23,has_touch:u8@24,offset:u8@25,tilt:i8@26,reservata:[u8;5]@27 |32/8 nos alvos CI64|
| SyRegimen | stilus:u8@0,strata:u8@1,reservata0:[u8;2]@2,tick_ms:u32@4,interval_ms:u32@8,layers:u8@12,lines:u8@13,reservata1:[u8;2]@14 |16/4|

`repr(C)` preserva alinhamento nativo: fora dos alvos64, especialmente ABI32, o header e Rust
DEVEM concordar pelo compilador; não impor alinhamento8 por pragma ou pack. Campos mantêm offsets.
Flags0/1; lingua0/1; stilus0..3; color_bits1/4/8/24; dimensões positivas;
strata1..4,lines0..6,tick_ms42/125/250/60000; interval_ms≥tick_ms e membro de INTERVALS;
reservados zero. tz_offset aceita todo i16 ABI, sem inventar restrição ausente no core.
Render exige width/height iguais a f e memória válida de pelo menos width*height bytes. Antes de formar slice, verificar multiplicação em usize e extensão≤isize::MAX; memória não nula inválida permanece violação de pré-condição do chamador.

- [ ] Escrever `ffi_roundtrip`: genesis→format32→parse→mesmas palavras; seed0 normalizado;
  version0x000100 para0.1.0; parse inválido preserva outputs; format não escreve byte33.
- [ ] Escrever `ffi_matches_core`: consilium,step,advance,render e palette48 equivalem ao
  core em cenários vigilia/quies/somnium/cells; input structs permanecem intactas.
- [ ] Escrever `ffi_invalid_parameters_no_write`: cada discriminante/flag/reservado inválido,
  dimensões incompatíveis, ticks65537,tick0,null em cada argumento exigido; void no-op,
  parse/step/advance retornam0. Não fabricar ponteiros inválidos não nulos como teste seguro.
- [ ] Escrever consumer C: `_Static_assert(sizeof/offsetof)` de todos os campos; sentinelas
  antes/depois dos arrays32/48/pixels; roundtrip e valores inválidos; chamada a todas as nove
  funções; comparar valores determinísticos com fixtures calculadas independentemente por Rust.
- [ ] Executar `cargo test -p sisyphium-limen`; observar falhas de exports ausentes.
- [ ] Implementar forma.rs segura com `TryFrom<Sy*>` e `From<Regimen>`; preencher reserved0.
  lib.rs contém unsafe extern C, checagem null, cópia das entradas, conversão e só então escrita;
  rustdoc `# Safety` registra validade, alinhamento, lifetime, extensão e não sobreposição.
  Ponteiros de outputs distintos não podem se sobrepor entre si nem a entradas.
- [ ] Manifesto `crate-type=["rlib","staticlib","cdylib"]`; não herdar lint unsafe forbid;
  repetir demais lints do workspace, unsafe_code=deny, unsafe_op_in_unsafe_fn=deny,
  undocumented_unsafe_blocks/missing_safety_doc=deny; allow unsafe apenas no limiar documentado. Versão por CARGO_PKG_VERSION
  parseada em const, cada componente≤255 verificado no build/teste, sem string fixa divergente.
- [ ] Implementar verify-ffi.py: construir crate, compilar consumer contra cdylib com cc/clang
  Unix e cl no ambiente MSVC em Windows; executar com caminho de biblioteca controlado;
  também linkar staticlib usando bibliotecas nativas reportadas por rustc, nunca supor rlib=C.
- [ ] Executar `python3 scripts/verify-ffi.py`; testes Rust+C passam. Revisar cada bloco unsafe
  com a pré-condição correspondente no header; core mantém forbid intacto.
- [ ] Commit `feat(limen): expose the documented c boundary`.

### Tarefa 5: scripts de wallpaper e agendas controladas

**Arquivos:** três scripts da tabela, `hosts/macos/sisyphium.plist.in`,
`hosts/linux/{sisyphium.service.in,sisyphium.timer.in}`, `hosts/windows/sisyphium-task.xml.in`,
`hosts/README.md`, `scripts/verify-hosts.py`; atualizar docs04§8 e docs08.
**Interface externa:** `dry-run|run|apply|install|uninstall`, padrão dry-run;
`--binary PATH --state-dir PATH --interval SECONDS --energy U8 --width U16 --height U16`;
Linux também `--backend feh|swww`. PowerShell usa os parâmetros equivalentes nomeados.
Intervalo padrão60, mínimo60; execução serial: timer/launchd/tarefa não sobrepõe instância;
transação de estado T1 protege também chamadas manuais. Dry-run não cria diretórios.

- [ ] Consultar fontes primárias atuais dos comandos escolhidos: pmset/osascript/launchd,
  xrandr/swaymsg/feh/swww/systemd e Get-CimInstance/SystemParametersInfo/Task Scheduler.
  Registrar URLs em docs/referencias.md e capabilities/fallbacks em hosts/README.md.
- [ ] Escrever `script_dry_run_no_mutation` com comandos falsos: comandos/argumentos ficam
  estruturados e escapados, nenhum aplicador/agendador executa; ausência de dependência aponta
  utilitário faltante, sem tentativa de instalação. Mesmos testes para os três sistemas no CI.
- [ ] Escrever `script_measurement_fallbacks`: energia ligada/desconhecida255, bateria
  floor(percent*255/100) com percent0..100; medida inválida→255; dimensões inválidas→320×180;
  flags explícitas prevalecem; nenhum valor não validado vira argumento de shell.
- [ ] Escrever `scheduler_scope`: caminho com espaços, aspas, &, $, Unicode; interval59
  recusado; install só escreve a agenda própria; uninstall não remove estado/imagens;
  fakes verificam chamada de apply apenas depois de PNG e state íntegros. Sem eval.
- [ ] Executar `python3 scripts/verify-hosts.py --platform current`; observar falhas esperadas.
- [ ] Implementar diretórios padrão: macOS `~/Library/Application Support/sisyphium`, Linux
  `${XDG_STATE_HOME:-$HOME/.local/state}/sisyphium`, Windows `%LOCALAPPDATA%\sisyphium`.
  Arquivos `mundus.hex` e `tabula.png`; run explícito pode criar apenas esse diretório.
- [ ] macOS mede pmset/sw_vers e dimensões por system_profiler; medida indisponível usa fallback;
  osascript recebe caminho por argv, nunca interpolado em código. launchd usa StartInterval
  e ProgramArguments; não registrar RunAtLoad oculto. Plist escapa XML.
- [ ] Linux mede power_supply e saída ativa de xrandr/swaymsg; escolhe feh com DISPLAY ou
  swww com WAYLAND_DISPLAY e comando disponível; nenhum backend→erro de apply. systemd usa
  OnUnitInactiveSec≥60, unidade oneshot, sem Persistent catch-up; escapar argumentos/systemd.
- [ ] Windows mede bateria/resolução; apply invoca SystemParametersInfoW via binding local
  documentado, verifica resultado; tarefa usa IgnoreNew e intervalo≥60. XML e argv escapados;
  sem política de execução global ou privilégios novos. Dependências .NET/PowerShell são do SO.
- [ ] Executar dry-run e run com fakes/diretório temporário; validar agendas por plutil,
  systemd-analyze verify ou parser XML/Task Scheduler fakes onde disponível. Não executar
  apply/install reais. Commit `feat(hosts): add controlled wallpaper adapters`.

### Tarefa 6: calibração empírica e fixação dos custos

**Arquivos:** `crates/sisyphium-tabula/src/metire.rs`, `scripts/calibrate-costs.py`,
`crates/sisyphium/tests/consilium_reference.rs` (ou adaptar teste independente já existente),
`crates/sisyphium/src/consilium.rs`, `docs/{06-consilium,08-verificacao}.md`;
fixtures determinísticas em `crates/sisyphium/tests/fixtures/consilium.tsv` se necessário.
**Interface:** `sisyphium-tabula --metire [--stilus NAME]` produz TSV em stdout, diagnóstico
em stderr; colunas cenário,stilus,largura,altura,strata,layers,lines,n,mediana_ns,p95_ns.
Sem arquivo de estado/saída; montar regimen experimental apenas dentro deste módulo.

- [ ] Escrever `measurement_excludes_conversion`: relógio delimita somente render; allocation,
  escolha,paleta,encode,hash e disco fora. Fixtures definem mundos/referência antes do intervalo.
- [ ] Escrever `calibration_integer_normalization`: entradas sintéticas base1000ns/delta250ns
  produzem base10/custo3 por ceil(10*delta/base); delta negativo→0; base0/coluna ausente falha.
  Todos os cálculos são inteiros, repetíveis; não ajustar amostra para obter número desejado.
- [ ] Medir os cinco cenários de docs08§6, quatro stili e dimensões320×180/1920×1080,
  1000/100 amostras respectivamente, com warm-up, black_box e sem build concorrente.
  Acrescentar pares base1estrato/0camadas/0linhas contra estratos2/3/4, cada camada isolada,
  ANNALES1..6 e acréscimos Littera/Bicolor. Ativar condições narrativas da camada medida.
- [ ] Para Littera/Bicolor, comparar com Creta no mesmo mundo, dimensões, strata, layers e lines; Creta/Musivum conservam acréscimo0. Para estratos/linhas, quantidade é a diferença do parâmetro entre o par, antes do ceil. Cada coeficiente é ceil(10*max(0,t_feature−t_base)/(t_base*quantidade)); base0 falha.
- [ ] Para cada coeficiente, usar maior mediana positiva dos deltas pareados normalizados
  pela base do mesmo cenário/stilus; estratos e annales dividem pela quantidade adicionada.
  Base10 ancora a escala existente; Creta/Musivum acréscimo0. Registrar dispersão e predição
  contra mediana de quadros completos; desvio>25% requer explicar não aditividade antes de fixar.
  Coeficiente>255 bloqueia packing atual: ampliar representação documentada, sem truncamento.
- [ ] Executar `cargo build --release -p sisyphium-tabula` e `target/release/sisyphium-tabula --metire`;
  capturar TSV e ambiente na evidência local do PR; script gera proposta, não edita core sozinho.
- [ ] Fixar somente BASE_COST/STRATUM_COST/LAYER_COSTS/STILUS_COSTS conforme evidência;
  preservar fórmula/budget/valores/EXP/forçamentos. Atualizar tabela docs06 e testes numéricos.
- [ ] Localizar modelo independente de cp3 antes de criar novo; incorporar corpus1568 já
  existente se disponível, mantendo implementação independente. Reexecutar contra todos os
  vetores após custos novos; fixture/regimen/hash só mudam por causa rastreável da calibração.
- [ ] Executar testes completos do núcleo e hosts, medições docs08 e comparação de modelo;
  registrar custo observado, versão, CPU, OS e limitações; não chamar duração de energia física.
- [ ] Commit `perf(consilium): calibrate host rendering costs` com tabela medida no corpo do PR.

### Tarefa 7: três sistemas, evidência e encerramento do checkpoint

**Arquivos:** `.github/workflows/ci.yml`, `scripts/verify.sh`, docs03/08 quando requisitos
apontam para provas novas, `README.md`; remover plano publicado somente no fechamento.
O controlador mantém ownership de documentação, lockfile, CI e integração final.

- [ ] Acrescentar Windows ao job verificatio mantendo Linux/macOS e minima1.85.0;
  fmt/clippy/test/doc/verba nos três. Usar shell bash para scripts existentes no runner Windows.
- [ ] Rodar verify-images.py, verify-ffi.py e verify-hosts.py em plataforma nativa de cada runner;
  Python padrão, toolchain C disponível explicitamente; não instalar dependências de runtime.
  Testes Windows chamam PowerShell; Unix testa shell. Staticlib e cdylib precisam executar C.
- [ ] Acrescentar smoke textus finito em todos; Unix subprocesso SIGTERM durante espera,
  depois ler state32 válido, lock liberado, e comparar termios antes/depois em PTY.
  Windows nativo: novo process group, GenerateConsoleCtrlEvent CTRL_BREAK e console controlado
  para CTRL_C, state32 salvo e callback removido; comparar console modes. Child::kill não
  substitui esses testes. Fechamento de pipe é cenário adicional, sem alegar cleanup abrupto.
- [ ] Executar `scripts/verify.sh` e os três verificadores locais; registrar saída exata,
  plataformas realmente executadas e limitações restantes. Corrigir falhas antes de ampliar teste.
- [ ] Medir em release, sem builds paralelos: render por família/stilus, advance65536 por
 11 repetições/mediana; `size_of<Mundus>()==16`; bytes e SHA-256 dos executáveis tabula/textus,
  staticlib/cdylib limen e rlib core separados. Artefato ausente não pode ser contado como zero.
- [ ] Renderizar via tabula e inspecionar os12 quadros-ouro e saída textus en/pt; só atualizar
  hashes se causa visual intencional estiver documentada. Preservar ouro raster com regimen fixo
  quando apenas custo mudou; registrar hashes novos de seleção separadamente.
- [ ] Revisão final do ramo: ABI unsafe/header, concorrência/mtime, PNG, terminal e scripts,
  custos/calibração, limites éticos e fontes. Resolver todos os findings materiais.
- [ ] Atualizar README para checkpoint4 no commit de fechamento; remover plano de docs/planos;
  verificar git diff/status, nenhum artefato local/medição bruto/lock incorporado por acidente.
- [ ] Abrir um PR do checkpoint a partir de main após merge anterior, com as seis seções de
  convencoes§5 e medições. Aguardar todos os checks obrigatórios e revisão independente.
- [ ] Merge por rebase; confirmar SHA de main e CI. Criar tag anotada `cp/4-hospites` nesse SHA,
  com evidência na mensagem; reler PR/tag remotamente. Só então declarar checkpoint concluído.

## Cobertura e autoverificação do plano

M2/PPM/PNG/scripts→T2/T5; M3/terminal→T3; M4/nove ABI→T4;
REQ-C-01..05/cache/adaptação→T1/T3/T6; H01/02→T2/T3/T4; H03→T4;
H04/ouros→T7; H05/medição→T6/T7; H06/tempo→T1/T2; D01..03→T1/T5/T7.
Todo artefato futuro tem dono, interface e prova. Os cinco focos de revisão estão associados
às tarefas respectivas. Pendências de ambiente devem ser registradas como limites de evidência;
não adiar calibração, scripts ou ABI para declarar este checkpoint completo.

Fontes de implementação PNG: [W3C](https://www.w3.org/TR/png-3/),
[RFC1950](https://www.rfc-editor.org/rfc/rfc1950.html),
[RFC1951](https://www.rfc-editor.org/rfc/rfc1951.html).
Sinais: [Apple signal](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/signal.3.html),
[Windows HandlerRoutine](https://learn.microsoft.com/en-us/windows/console/handlerroutine).
As notas locais signal-contracts/atomic-file-contracts preservam evidência e limitações;
a implementação integra as referências pertinentes em docs/referencias.md.
