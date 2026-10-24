# ADR 0008 — Fronteiras nativas dos hospedeiros

- **Status:** aceito
- **Relacionados:** `03-especificacao.md` REQ-H-03 e REQ-D-02; `04-arquitetura.md` §3.3 e §4; ADR 0001 e ADR 0003

## Contexto

O núcleo é total, sem dependências, `no_std` e sem código inseguro. Os hospedeiros nativos
precisam traduzir ponteiros de uma ABI C, receber sinais do sistema e persistir o estado.
Essas operações exigem contratos que não pertencem à simulação.

## Decisão

O workspace mantém `unsafe_code = forbid`. Núcleo, `sisyphium-hospes` e `tabula` herdam essa
restrição. Somente `limen` e `textus` deixam de herdar os lints; seus manifestos repetem todos
os demais lints e usam `unsafe_code = deny`, `unsafe_op_in_unsafe_fn = deny`,
`clippy::undocumented_unsafe_blocks = deny` e `clippy::missing_safety_doc = deny`.

As permissões locais abrangem somente funções ABI de `limen` e o módulo `signa` de `textus`.
Cada bloco registra sua pré-condição. A ABI valida null, discriminantes, reservados e extensões
calculáveis antes de escrever. O chamador garante validade, alinhamento, duração, extensão e
não sobreposição da memória. A representação C é separada da representação Rust do núcleo.

O tratamento de sinais admite Linux glibc e macOS de 64 bits em x86_64 e aarch64, com
SIGINT e SIGTERM, e Windows com CTRL_C e CTRL_BREAK. O callback apenas grava `AtomicBool`
com ordenação relaxada; não executa E/S, alocação ou exclusão mútua. A plataforma DEVE possuir
atômicos de 8 bits. O laço consulta a marca e salva em contexto normal. A restauração Unix
preserva a disposição anterior de `signal`, sem promessa sobre máscara ou flags de `sigaction`.
Fechamento, logoff, shutdown, SIGKILL e TerminateProcess não garantem salvamento.

`sisyphium-hospes` compartilha exclusivamente argumentos, tempo, cache de seleção e
persistência entre os executáveis. A persistência usa APIs seguras da biblioteca padrão:
lock vazio por `create_new`, fotografia de bytes e `mtime`, temporário no mesmo pai,
`sync_all`, fechamento, `rename` e sincronização do diretório em Unix. O arquivo anterior
nunca é apagado antes da substituição. Escritores respeitam o mesmo protocolo; estado
corrompido, symlink ou conflito causa erro explícito.

Um erro após `rename` pode coexistir com o novo estado publicado. Esse erro exige encerramento
sem repetição nem restauração automática. Locks órfãos requerem recuperação manual. A
fotografia não elimina o retorno aos mesmos bytes e `mtime`. A proteção não cobre alteração
hostil de caminhos ou semântica de sistemas de arquivos remotos.

## Alternativas

Proibir toda fronteira insegura também nos hospedeiros impede a ABI e os callbacks nativos.
Permitir código inseguro em todo o workspace enfraquece a restrição do núcleo. Uma dependência
externa para encapsular essas mesmas fronteiras amplia o conjunto de código sem eliminar as
pré-condições do chamador. A permissão local mantém a extensão verificável da exceção.

Escrever diretamente sobre o estado expõe conteúdo parcial. Apagar antes de renomear cria
uma janela sem estado. Um histórico ou relógio auxiliar viola o formato persistido. O
protocolo escolhido conserva os 32 bytes e explicita seus limites.

## Consequências

- A verificação de ponteiros e sinais fica confinada a duas fronteiras documentadas.
- O núcleo mantém as restrições e a representação de 128 bits.
- Escritores cooperantes detectam alterações de fotografia antes de substituir o estado.
- A substituição local do nome em Unix é atômica; a sincronização solicitada não promete
  durabilidade absoluta após perda de energia nem equivalência em todos os sistemas de arquivos.
- Um erro pode representar publicação já concluída com confirmação ou limpeza falha.

## Fontes

- [Rust: criação exclusiva de arquivo](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.create_new).
- [Rust: substituição por rename](https://doc.rust-lang.org/std/fs/fn.rename.html).
- [Rust: sincronização de arquivo](https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_all).
- [Linux: rename e limites de sistemas de arquivos remotos](https://man7.org/linux/man-pages/man2/rename.2.html).
- [Linux: sincronização do diretório](https://man7.org/linux/man-pages/man2/fsync.2.html).
- [macOS: fsync e sincronização física](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fsync.2.html).
- [Windows: callback de controle do console](https://learn.microsoft.com/en-us/windows/console/handlerroutine).
