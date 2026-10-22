# ADR 0006 — Línguas, vocabulário latino e discrição de autoria

- **Status:** aceito
- **Relacionados:** `convencoes.md` §1–3; `02-ontologia.md` §3

## Contexto

A obra pretende canonicidade entre culturas e épocas, discrição de autoria e ausência de
vocabulário de produtos. O meio técnico (Rust, commits) tem convenções em inglês. A língua do
autor é o português.

## Decisão

Nome próprio e vocabulário externo em latim: nome do projeto, crates, módulos, tipos, campos de
estado e termos definidos. Documentação em português com linguagem controlada. Instruções de
código (funções, variáveis, documentação de API, mensagens de commit) em inglês. Texto da
experiência em inglês e português, selecionável. Nenhuma menção a pessoas, ferramentas ou
processos de produção; autoria apenas na conta do repositório.

## Consequências

- O glossário é a fonte dos nomes; um termo novo entra primeiro no glossário.
- Os textos da obra existem em dois corpora autorais equivalentes em registro.
- Commits, PRs e releases carregam o histórico; os documentos descrevem só o presente.
