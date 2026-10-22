# Clinamen Sisyphium

Fundo de tela dinâmico. Uma simulação de 128 bits encena a ascensão repetida de Sísifo sob leis
derivadas do ambiente que a executa: a energia disponível é a temperatura do mundo; o relógio é a
efeméride; a tela é a geometria; a interação é o sismo; a memória é uma única palavra de 64 bits.
O valor da obra é o pensamento que a imagem exige. Cada traço decorre de uma teoria nomeada, de
mais de uma cultura e época, e de uma lei verificável.

O nome une o termo de Lucrécio para o desvio mínimo que rompe o destino, _clinamen_ (_De rerum
natura_, II, 216–293), ao adjetivo latino do mito (_Sisyphius_: Propércio, II, 17; Ovídio,
_Metamorfoses_, IV, 460). No núcleo, o clinamen é um micro-incidente com três ramos, disparado em tempo e lugar
determinados pelo gerador caótico; a lasca que inverte um bit da rocha é outro evento (_lapsus_). O mito é o canal de toda a filosofia apresentada. Forma curta
nos crates e no ABI: `sisyphium`.

## Estado

Checkpoint presente: 2 (signo e corpo: `concentus`, `signum` com pixel, raster, fonte, céu,
encosta, moldura, compositor e quatro stili; `corpus`; quadros-ouro). Evidências de cada checkpoint estão no PR correspondente e na
mensagem da tag `cp/N-assunto`.

## Verificação

```
scripts/verify.sh
```

Executa formato, lints, testes, documentação e varredura de termos vetados, na ordem de
`docs/08-verificacao.md` §3.

## Mapa dos documentos

| Documento | Conteúdo |
| --- | --- |
| `docs/00-metacognicao.md` | Pré-planejamento: abstratos de valor, essencial e acidental, sinergia |
| `docs/01-visao.md` | Harness irrefutável, intenção, metas, não-metas, antimetas, restrições, decisões |
| `docs/02-ontologia.md` | Níveis ontológicos, teorias por nível e cultura, glossário de termos definidos |
| `docs/03-especificacao.md` | Requisitos numerados com método de verificação e rastreabilidade |
| `docs/04-arquitetura.md` | Crates, módulos, contrato de hospedeiro, formatos de quadro e de estado |
| `docs/05-estetica.md` | Doutrina estética: linha orgânica, stili, concentus, constantes de composição |
| `docs/06-consilium.md` | Meta-heurística de adaptação ao meio e à situação narrativa |
| `docs/07-narrativa.md` | Cânone narrativo, corpus e regras de escrita |
| `docs/08-verificacao.md` | Estratégia de verificação, medições e integração contínua |
| `docs/convencoes.md` | Línguas, estilo, densidade canônica, commits, PRs, versionamento |
| `docs/referencias.md` | Bibliografia canônica |
| `docs/adr/` | Decisões de arquitetura |
| `docs/planos/` | Planos de implementação por checkpoint |

## Direitos

Código, textos e imagens são protegidos pela Lei nº 9.610/1998 e pela Lei nº 9.609/1998. O
repositório é público para estudo. Reprodução, distribuição e obras derivadas são reservadas. O
texto integral está em `LICENSE`.
