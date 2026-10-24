//! The chronicles in Portuguese: eight clinamina × three branches × five lines, one line per
//! register, and the lines of the reserved events (`docs/07-narrativa.md` §8).

/// Chronicle lines by `eventus`, `ramus` and line index; line i belongs to register i.
pub const ANNALES: [[[&str; 5]; 3]; 8] = [
    // 0 folium
    [
        [
            "A FOLHA DESLIZA PARA A POEIRA.",
            "UMA CORUJA ACORDA NA OLIVEIRA.",
            "O VOO DERRUBA UMA LÂMPADA EM ATENAS.",
            "OS ROLOS SOBRE A CLEMÊNCIA ARDEM.",
            "FALTA UMA PALAVRA PARA CLEMÊNCIA.",
        ],
        [
            "A FOLHA ESTALA SOB A SANDÁLIA.",
            "UM GRILO SALTA COM O SOM.",
            "UM CARRO DE SEDA DESVIA EM CHANG'AN.",
            "A TINTA DO MANTO IMPERIAL ENTORNA.",
            "UMA DINASTIA DATA A QUEDA DESSE ANO.",
        ],
        [
            "A FOLHA FICA INTEIRA NA PEDRA.",
            "UMA FORMIGA LEVA A FOLHA AO NINHO.",
            "LAVRADOR DE TENOCHTITLAN FIRMA O SOLO.",
            "O MILHO NUTRE A CIDADE DO LAGO UM ANO.",
            "A ORDEM DAS COISAS FICA IMÓVEL.",
        ],
    ],
    // 1 calculus
    [
        [
            "O SEIXO MÓI O CALCANHAR A CADA PASSO.",
            "O SEIXO POLIDO ROLA PARA FORA.",
            "UMA TRUTA DESVIA. UM ANZOL ESPERA.",
            "O PESCADOR FECHA AS PORTAS DE TROIA.",
            "SEM LINHAGEM DE ENEIAS. SEM ROMA.",
        ],
        [
            "A SANDÁLIA ESVAZIA. A PEDRA CAI.",
            "A PEDRA BATE NUMA COLMEIA DA ESCARPA.",
            "O ENXAME DISPERSA A CARAVANA DE PÚRPURA.",
            "SEM TINTA PARA O MANTO EM PERSÉPOLIS.",
            "UM GENERAL LÊ O ATRASO COMO FRAQUEZA.",
        ],
        [
            "O CALCANHAR MÓI A PEDRA EM PÓ.",
            "UMA RAJADA QUENTE ERGUE O PÓ AO CÉU.",
            "CAI GRANIZO NA CEVADA DE CROTONA.",
            "A ESCOLA DOS NÚMEROS PASSA FOME.",
            "O INFINITO ESPERA DOZE SÉCULOS.",
        ],
    ],
    // 2 umbra
    [
        [
            "A SOMBRA CRUZA A PEDRA.",
            "UM MILHAFRE GIRA UMA VEZ E VAI.",
            "UM ESCRIBA EM PATALIPUTRA TAMBÉM OLHA.",
            "O ÉDITO DA CLEMÊNCIA SAI EM DUAS CÓPIAS.",
            "A SEGUNDA CÓPIA SOBREVIVE AO FOGO.",
        ],
        [
            "A SOMBRA PASSA SEM OLHAR.",
            "A AVE POUSA NO TELHADO DE UM FUNDIDOR.",
            "O BRONZE ESFRIA UM SOPRO ANTES.",
            "UMA PLACA DO BENIM GUARDA UMA RACHA.",
            "A RACHA É LIDA COMO SINAL POR ERAS.",
        ],
        [
            "A PEDRA ESCORREGA UM PALMO.",
            "UM LAGARTO FOGE DA FENDA.",
            "UM CAMELO EMPACA À PORTA DE TIMBUKTU.",
            "UM FARDO DE SAL APODRECE NO MURO.",
            "UMA ESCOLA FECHA POR FALTA DE SAL.",
        ],
    ],
    // 3 sudor
    [
        [
            "A GOTA É ENXUTA ANTES DE CAIR.",
            "A PEDRA FICA SECA E LEVE.",
            "UM NÓ É ATADO NUM QUIPU EM CUZCO.",
            "A CONTA DO MILHO SAI EXATA.",
            "NENHUMA FOME É ESCRITA NESSE ANO.",
        ],
        [
            "A GOTA ESCURECE UM PONTO DA PEDRA.",
            "UMA ABELHA PROVA O SAL E VAI.",
            "UM JARDIM DE ALEXANDRIA PERDE ABELHAS.",
            "OS FIGOS DA BIBLIOTECA NÃO AMADURECEM.",
            "UM COPISTA PASSA FOME. UM LIVRO ESPERA.",
        ],
        [
            "O SAL ACORDA A LÍNGUA.",
            "O PASSO ACELERA POR UM TEMPO.",
            "UM NAVIO DEIXA EDO NA MARÉ DA MANHÃ.",
            "UM BAÚ DE CHÁ CHEGA AO NORTE.",
            "UM HÁBITO DE CHÁ ATRAVESSA UMA ERA.",
        ],
    ],
    // 4 rima
    [
        [
            "O PÉ TESTA A RACHA E AGUENTA.",
            "UM BESOURO SAI DA FENDA.",
            "UM OLEIRO DE URUK IMPRIME O BESOURO.",
            "O SINAL CONTA JARROS, DEPOIS ANOS.",
            "A ESCRITA COMEÇA NUMA RACHA.",
        ],
        [
            "O PÉ PULA A RACHA.",
            "A POEIRA ASSENTA NA RACHA INTOCADA.",
            "UM AGRIMENSOR DE CARTAGO LÊ PRESSÁGIO.",
            "O MURO DO PORTO SAI UM ANO ATRASADO.",
            "UMA FROTA SURGE DIANTE DA CIDADE ABERTA.",
        ],
        [
            "O CALCANHAR ALARGA A RACHA EM FENDA.",
            "A FENDA BEBE A PRIMEIRA CHUVA.",
            "EM KYOTO UMA CISTERNA FICA VAZIA.",
            "UM TEMPLO ARDE SEM ÁGUA POR PERTO.",
            "OS SUTRAS SOBEM COM O TELHADO.",
        ],
    ],
    // 5 formica
    [
        [
            "O PÉ ESPERA A FORMIGA PASSAR.",
            "A SEMENTE CHEGA AO NINHO.",
            "A FIGUEIRA SOMBREIA UM PÁTIO EM NALANDA.",
            "UM MONGE LÊ MAIS UMA PÁGINA À SOMBRA.",
            "UMA PROVA SOBRE O ZERO VENCE AS CHUVAS.",
        ],
        [
            "O PÉ CONTORNA A FORMIGA.",
            "A FORMIGA MUDA DE RUMO E LARGA O GRÃO.",
            "UMA MUDA CRESCE NO CAMINHO DE AXUM.",
            "CARAVANAS PARAM ALI UM SÉCULO DEPOIS.",
            "UM MERCADO GUARDA O NOME DE UMA ÁRVORE.",
        ],
        [
            "A FORMIGA MORRE SOB UM PASSO CEGO.",
            "A SEMENTE NUNCA BROTA.",
            "EM RAPA NUI UMA PALMEIRA A MENOS CRESCE.",
            "A MADEIRA DA ÚLTIMA CANOA ACABA ANTES.",
            "UMA ILHA ESQUECE O MAR.",
        ],
    ],
    // 6 echo
    [
        [
            "UM GRITO DESCE AO VALE.",
            "UMA CABRA FOGE E PERDE O SINO.",
            "CONSTANTINOPLA: PASTOR MIRIM ACHA MOEDA.",
            "A MOEDA PAGA UM LUGAR NUMA ESCOLA.",
            "UM CÓDIGO DE LEIS SOBREVIVE AO IMPÉRIO.",
        ],
        [
            "O ECO MORRE SEM RESPOSTA.",
            "UMA LEBRE FICA QUIETA NO CAPIM.",
            "UM CAÇADOR DE BAGDÁ VOLTA SEM NADA.",
            "A NOITE VAI PARA UM PROBLEMA.",
            "A ÁLGEBRA GANHA UMA NOITE.",
        ],
        [
            "O ECO VOLTA MUDADO.",
            "UMA SEGUNDA VOZ VEM DENTRO DO SOM.",
            "EM TEOTIHUACAN UMA FLAUTA ACHA UM TOM.",
            "O TOM ABRE UMA FESTA A CADA PRIMAVERA.",
            "UMA CIDADE MARCA O TEMPO POR UM SOM.",
        ],
    ],
    // 7 ventus
    [
        [
            "O CORPO PENDE CONTRA A RAJADA.",
            "UMA SEMENTE DE CARDO PASSA PELA ORELHA.",
            "UM CARDO CRIA RAIZ EM CÓRDOBA.",
            "UM JARDINEIRO APRENDE UM ENXERTO.",
            "LARANJAS CHEGAM A UM PAÍS FRIO.",
        ],
        [
            "A SUBIDA ESPERA O VENTO PASSAR.",
            "UMA GAIVOTA VAI NA RAJADA PARA O MAR.",
            "UMA CANOA DE TONGA LÊ A LINHA DA AVE.",
            "A CANOA ACHA UMA ILHA PELA GAIVOTA.",
            "UMA ROTA É CANTADA POR VINTE GERAÇÕES.",
        ],
        [
            "O ROSTO DESVIA DA POEIRA.",
            "UMA MARIPOSA ENTRA NO COLARINHO.",
            "A MARIPOSA VAI NUMA BARCAÇA PARA ANGKOR.",
            "UMA SAFRA DE SEDA FALHA UMA ESTAÇÃO.",
            "UM TEMPLO PERDE UMA DÉCADA.",
        ],
    ],
];

/// One aphorism per clinamen, drawn at its branching node (REQ-L4-12).
pub const APHORISMI: [&str; 8] = [
    "□FOLHA → ◊FOGO",
    "□PEDRA → ◊ROMA",
    "□SOMBRA → ◊SINAL",
    "□GOTA → ◊CONTA",
    "□RACHA → ◊LETRA",
    "□GRÃO → ◊ÁRVORE",
    "□ECO → ◊LEI",
    "□VENTO → ◊ROTA",
];

/// The lines of Diogenes, Heraclitus and the daemon (`eventus` 13..=15).
pub const PERSONAE: [&str; 3] = [
    "POR QUE CARREGAR UMA PEDRA SEM DONO?",
    "A MESMA PEDRA NÃO SOBE DUAS VEZES.",
    "ESTA SUBIDA DE NOVO, E DE NOVO, SEM FIM?",
];

/// The two lines of the recognition (`eventus` 12): intimate, then abstract.
pub const AGNITIO: [&str; 2] = [
    "A CABEÇA REPOUSA NA PEDRA FRIA.",
    "A SUBIDA É SONHO. A PEDRA NÃO.",
];

/// The line of the metalepsis (`eventus` 11), abstract.
pub const METALEPSIS: &str = "O CICLO MOVE A MÃO.";

/// The lines of the interruption: two epochal, then two civil.
pub const INTERRUPTIO: [&str; 4] = [
    "A MÃO MEXE. A GUERRA ESPERA.",
    "UM CURSOR DETÉM UM SÉCULO.",
    "UMA PORTA DE TROIA FICA ENTREABERTA.",
    "UMA CARAVANA ACAMPA NUM POÇO LACRADO.",
];
