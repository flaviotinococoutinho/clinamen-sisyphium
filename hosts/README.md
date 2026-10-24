# Hospedeiros do desktop

Os adaptadores executam `sisyphium-tabula` e controlam a publicação de uma imagem estática.
Cada execução delega evolução, escolha e persistência ao binário. Os scripts NÃO DEVEM
interpretar o estado do mundo nem substituir as leis do núcleo.

## Ações e argumentos

| Ação | Efeito |
| --- | --- |
| `dry-run` | Padrão. Sonda capacidades e imprime o comando escapado. Não executa o binário, não cria diretórios e não chama o aplicador ou o agendador. |
| `run` | Cria o diretório de estado, executa o binário e verifica os arquivos publicados. |
| `apply` | Executa `run` e solicita a aplicação da imagem após retorno zero e validação dos arquivos. |
| `install` | Registra somente a agenda própria. A agenda chama `apply` após o intervalo configurado. |
| `uninstall` | Desregistra somente a agenda própria. Preserva imagem e estado. |

macOS e Linux recebem a ação como primeiro argumento. A ausência da ação seleciona
`dry-run`. Windows recebe `-Action`; a ausência desse parâmetro seleciona `dry-run`.

| Shell | PowerShell | Contrato |
| --- | --- | --- |
| `--binary PATH` | `-Binary PATH` | Executável existente. Padrão: `target/release/sisyphium-tabula`, com extensão `.exe` no Windows, relativo à raiz do projeto. |
| `--state-dir PATH` | `-StateDir PATH` | Diretório que contém `mundus.hex` e `tabula.png`. |
| `--interval SECONDS` | `-Interval SECONDS` | Inteiro em `60..=2147483647` segundos; padrão 60 segundos. Instalação Windows limita o intervalo a 2678400 segundos, conforme Task Scheduler. |
| `--energy U8` | `-Energy U8` | Inteiro em `0..=255`. Prevalece sobre a sondagem. |
| `--width U16` | `-Width U16` | Inteiro em `1..=65535` pixels. Prevalece sobre a largura sondada. |
| `--height U16` | `-Height U16` | Inteiro em `1..=65535` pixels. Prevalece sobre a altura sondada. |
| `--lingua en\|pt` | `-Lingua en\|pt` | Língua da experiência. Padrão `en`. |
| `--backend feh\|swww` | — | Exclusivo do Linux. Seleciona o aplicador. |

A área NÃO DEVE exceder 33554432 pixels. Argumento inválido retorna 2; falha operacional
retorna 1; sucesso retorna 0. Opções repetidas ou desconhecidas falham. O diretório de
estado e o binário NÃO DEVEM ser caminhos vazios. `uninstall` não exige binário existente.

Exemplos de sondagem, a partir da raiz do projeto:

```sh
sh hosts/macos/sisyphium.sh dry-run --energy 255 --width 320 --height 180 --lingua pt
sh hosts/linux/sisyphium.sh dry-run --backend feh --lingua en
```

```powershell
& ./hosts/windows/sisyphium.ps1 -Action dry-run -Energy 255 -Width 320 -Height 180 -Lingua pt
```

A alteração explícita da ação para `run` publica os arquivos. `apply` altera o wallpaper.
`install` autoriza aplicações periódicas. Nenhuma ação instala dependências ou compila o
binário. A política de execução do PowerShell permanece sob controle do sistema.

## Publicação e caminhos

| Sistema | Diretório padrão |
| --- | --- |
| macOS | `~/Library/Application Support/sisyphium` |
| Linux | `${XDG_STATE_HOME:-$HOME/.local/state}/sisyphium` |
| Windows | `%LOCALAPPDATA%\sisyphium` |

O comando emitido contém `--state <diretório>/mundus.hex`, `--output <diretório>/tabula.png`,
`--format png`, `--energy`, `--width`, `--height` e `--lingua`. Argumentos permanecem
separados; o texto de diagnóstico nunca é avaliado como código.

O diretório DEVE ser privado do usuário. O protocolo de persistência do binário controla
escritores cooperantes por lock e comparação do estado lido. Os scripts não removem locks
órfãos, não repetem uma execução com falha e não prometem proteção contra alterações
externas concorrentes. As restrições da persistência constam de
[arquitetura](../docs/04-arquitetura.md).

Antes da aplicação, o adaptador exige retorno zero do binário, estado regular de exatamente
32 dígitos hexadecimais sem newline e imagem regular com assinatura PNG de oito bytes.
Links simbólicos nas duas publicações são recusados. Essa verificação de assinatura não é
uma decodificação PNG completa. Uma falha do aplicador pode ocorrer após a publicação dos
arquivos; o adaptador não restaura o estado ou o wallpaper anterior.

Espaços, aspas, `&`, `$`, `%` e Unicode são argumentos literais. Shell conserva caminhos
com bytes não UTF-8 nas operações em que o sistema de arquivos os admite. O sistema PODE
recusar esses nomes. Agendas Unix exigem caminhos UTF-8 sem caracteres de controle, pois
XML e arquivos de unidade têm restrições textuais. Windows usa caminhos nativos Unicode;
caracteres proibidos pelo sistema de arquivos não constituem nomes válidos.

## Sondagem e aplicação

| Sistema | Energia | Dimensões | Aplicação explícita |
| --- | --- | --- | --- |
| macOS | `pmset -g batt`; descarga válida em `0..=100` é convertida por `floor(percentual * 255 / 100)`; demais resultados usam 255. | Primeira resolução válida de `system_profiler SPDisplaysDataType`; `sw_vers` consulta a versão sem impor versão mínima. | `osascript` recebe o caminho por argv. AppleScript constante solicita a imagem para cada desktop de System Events. |
| Linux | Primeiro `power_supply` em descarga com capacidade válida; mesma conversão inteira. Ausência, alimentação externa ou leitura inválida usam 255. | Primeira saída ativa de `xrandr --current`, ou primeiro modo de `swaymsg -p -t get_outputs` em sessão Wayland. | `feh --no-fehbg --bg-fill` em X11; `swww img --transition-type none` em Wayland. |
| Windows | `Get-CimInstance Win32_Battery`; estado de descarga e capacidade válida usam a conversão inteira. Demais resultados usam 255. | Primeira resolução válida de `Win32_VideoController`. | Binding local de `SystemParametersInfoW`, ação 20, flags 3. O retorno da API é verificado. |

Dimensões ausentes, inválidas ou acima do limite de área usam 320 × 180 pixels. Flags
explícitas prevalecem individualmente; a combinação final também respeita o limite de área.
Nenhuma medida não validada integra uma expressão de shell.

No macOS, `dry-run` e `run` não consultam System Events. `apply` depende das permissões de
automação do sistema e PODE provocar uma solicitação de autorização. A agenda não concede
permissões de automação. No Linux, a seleção automática prefere `swww` quando há
`WAYLAND_DISPLAY` e executável disponível; em seguida considera `feh` com `DISPLAY`.
Ausência de aplicador ou sessão compatível produz diagnóstico. O daemon de `swww` DEVE
estar ativo; o script não o inicia. A sessão do agendador DEVE disponibilizar as variáveis
gráficas e os executáveis necessários. As sondagens ausentes usam os valores de fallback.

Os adaptadores usam shell ou PowerShell e utilitários do sistema. Python não é dependência
de execução dos adaptadores. As fontes dos contratos estão em
[referências](../docs/referencias.md#integração-com-o-desktop).

## Agendas

| Sistema | Identidade e arquivos | Cadência e serialização |
| --- | --- | --- |
| macOS | `org.sisyphium.tabula`; `~/Library/LaunchAgents/org.sisyphium.tabula.plist` | `StartInterval` e `ProgramArguments` em array. Não registra `RunAtLoad`. Uma instância por serviço launchd. |
| Linux | `sisyphium.service` e `sisyphium.timer` em `${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user` | Serviço `oneshot`; primeira ativação por `OnActiveSec`, repetições por `OnUnitInactiveSec`. Sem `Persistent`. Serviço ativo não recebe nova instância. |
| Windows | Tarefa `Sisyphium.Tabula` na raiz do Task Scheduler | Primeiro início após o intervalo; repetição XML com `IgnoreNew`. Token interativo do usuário e menor privilégio. Sem recuperação de execuções perdidas. |

A instalação conserva flags explícitas de energia e dimensões; medidas automáticas são
recalculadas em cada execução agendada. O caminho do adaptador permanece referenciado pela
agenda. O diretório do projeto NÃO DEVE ser movido enquanto a agenda estiver registrada.

Arquivos de agenda preexistentes não são sobrescritos. A desinstalação exige o marcador
próprio, interrompe a agenda correspondente e preserva outros arquivos. O marcador identifica
o escopo operacional; não constitui autenticação contra alterações externas. Falha parcial
de registro deixa a definição disponível para inspeção. Uma falha ao desregistrar preserva
a definição. Estado e imagem não são apagados.

XML escapa os caracteres reservados. Unidades systemd escapam aspas, barras, `%` e `$`.
Task Scheduler recebe argumentos conforme as regras da linha de comando Windows. Templates
adjacentes aos scripts definem somente essas agendas. Não há agenda criada implicitamente
por `dry-run`, `run` ou `apply`.

## Verificação isolada

```sh
python3 scripts/verify-hosts.py --platform current
```

`--platform unix` verifica ambos os adaptadores shell com comandos falsos. `--platform
windows` exige PowerShell. O verificador não demonstra aplicação nativa de wallpaper.
Os contratos de argumentos, fallback, publicação, ausência de dependências e escopo das
agendas são exercitados com executáveis ou funções substitutas.

O verificador fixa hashes dos scripts e templates revisados, copia os arquivos e confere
novamente os hashes antes da execução. Mudanças em fronteiras de comando exigem revisão
dos hashes. O `PATH` contém somente substitutos e utilitários inertes verificados. Um
preflight de resolução exige que aplicadores ausentes permaneçam não resolvidos. O
ambiente não herda diretórios pessoais ou hooks de shell. Windows verifica a origem das
funções substitutas e usa uma classe gerenciada sem chamada à API de desktop.

A evidência fica em `out/host-tests`, inclusive em falhas. `SISYPHIUM_HOST_TEST_ROOT` ou
`CARGO_TARGET_TMPDIR` seleciona outra raiz. Cada execução cria um diretório privado e
mantém `execution.jsonl`, registros dos comandos falsos e `result.json`. Os registros
contêm caminhos locais e DEVEM permanecer fora de artefatos públicos. O verificador não
remove automaticamente essas evidências. `plutil` e `systemd-analyze verify` constituem
validações externas somente leitura; o verificador não os executa nativamente.
