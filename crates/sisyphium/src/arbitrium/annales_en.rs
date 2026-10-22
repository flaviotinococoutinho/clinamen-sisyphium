//! The chronicles in English: eight clinamina × three branches × five lines, one line per
//! register, and the lines of the reserved events (`docs/07-narrativa.md` §8).

/// Chronicle lines by `eventus`, `ramus` and line index; line i belongs to register i.
pub const ANNALES: [[[&str; 5]; 3]; 8] = [
    // 0 folium
    [
        [
            "THE LEAF SKIDS INTO THE DUST.",
            "AN OWL WAKES IN THE OLIVE TREE.",
            "THE FLIGHT TIPS A LAMP IN ATHENS.",
            "THE SCROLLS ON MERCY BURN TONIGHT.",
            "A WORD FOR MERCY IS LOST FOR AGES.",
        ],
        [
            "THE LEAF CRACKS UNDER THE SANDAL.",
            "A CRICKET LEAPS FROM THE SOUND.",
            "A SILK CART SWERVES NEAR CHANG'AN.",
            "THE DYE FOR THE IMPERIAL ROBE SPILLS.",
            "A DYNASTY DATES A FALL FROM THAT YEAR.",
        ],
        [
            "THE LEAF STAYS WHOLE ON THE STONE.",
            "AN ANT CARRIES THE LEAF TO THE NEST.",
            "THE SOIL OF A CHINAMPA HOLDS.",
            "MAIZE FEEDS A LAKE CITY ONE MORE YEAR.",
            "THE ORDER OF THINGS STAYS UNMOVED.",
        ],
    ],
    // 1 calculus
    [
        [
            "THE PEBBLE GRINDS THE HEEL AT EACH STEP.",
            "THE PEBBLE POLISHES AND ROLLS OUT.",
            "A TROUT TURNS AWAY. A HOOK WAITS.",
            "THE FISHER LOCKS THE GATES OF TROY.",
            "NO LINE OF AENEAS. NO ROME.",
        ],
        [
            "THE SANDAL EMPTIES. THE STONE FALLS.",
            "THE STONE STRIKES A HIVE ON THE LEDGE.",
            "THE SWARM SCATTERS A PURPLE CARAVAN.",
            "NO DYE FOR THE ROBE AT PERSEPOLIS.",
            "A GENERAL READS THE DELAY AS WEAKNESS.",
        ],
        [
            "THE HEEL GRINDS THE STONE TO DUST.",
            "A WARM GUST LIFTS THE DUST SKYWARD.",
            "HAIL FALLS ON THE BARLEY OF CROTON.",
            "THE SCHOOL OF NUMBERS GOES HUNGRY.",
            "THE INFINITE WAITS TWELVE CENTURIES.",
        ],
    ],
    // 2 umbra
    [
        [
            "THE SHADOW CROSSES THE STONE.",
            "A KITE CIRCLES ONCE AND LEAVES.",
            "A SCRIBE IN PATALIPUTRA LOOKS UP TOO.",
            "THE EDICT ON MERCY IS COPIED TWICE.",
            "A SECOND COPY OUTLIVES THE FIRE.",
        ],
        [
            "THE SHADOW PASSES UNSEEN.",
            "THE BIRD LANDS ON A SMELTER'S ROOF.",
            "THE BRONZE COOLS A BREATH TOO SOON.",
            "A PLAQUE OF BENIN KEEPS A CRACK.",
            "THE CRACK IS READ AS A SIGN FOR AGES.",
        ],
        [
            "THE STONE SLIPS A HAND'S WIDTH.",
            "A LIZARD DARTS FROM THE CRACK.",
            "A CAMEL BALKS AT THE GATE OF TIMBUKTU.",
            "A BALE OF SALT ROTS AT THE WALL.",
            "A SCHOOL CLOSES FOR LACK OF SALT.",
        ],
    ],
    // 3 sudor
    [
        [
            "THE DROP IS WIPED BEFORE THE FALL.",
            "THE STONE STAYS DRY AND LIGHT.",
            "A KNOT IS TIED ON A QUIPU IN CUZCO.",
            "THE COUNT OF MAIZE COMES OUT EXACT.",
            "NO FAMINE IS WRITTEN FOR THAT YEAR.",
        ],
        [
            "THE DROP DARKENS A SPOT OF STONE.",
            "A BEE TASTES THE SALT AND LEAVES.",
            "A GARDEN OF ALEXANDRIA LOSES THE BEES.",
            "THE FIGS OF THE LIBRARY DO NOT RIPEN.",
            "A COPYIST GOES HUNGRY. A BOOK WAITS.",
        ],
        [
            "THE SALT WAKES THE TONGUE.",
            "THE STEP QUICKENS FOR A WHILE.",
            "A SHIP LEAVES EDO ON THE MORNING TIDE.",
            "A CHEST OF TEA REACHES THE NORTH.",
            "A HABIT OF TEA SPREADS ACROSS AN AGE.",
        ],
    ],
    // 4 rima
    [
        [
            "THE FOOT TESTS THE CRACK AND HOLDS.",
            "A BEETLE CRAWLS OUT OF THE SPLIT.",
            "A POTTER IN URUK PRESSES A BEETLE SIGN.",
            "THE SIGN COUNTS JARS, THEN YEARS.",
            "WRITING BEGINS WITH A CRACK.",
        ],
        [
            "THE FOOT SKIPS THE CRACK.",
            "THE DUST SETTLES ON THE CRACK UNTOUCHED.",
            "A SURVEYOR OF CARTHAGE READS AN OMEN.",
            "THE HARBOR WALL IS BUILT A YEAR LATE.",
            "A FLEET SAILS AGAINST AN OPEN CITY.",
        ],
        [
            "THE HEEL WIDENS THE CRACK TO A SPLIT.",
            "THE SPLIT DRINKS THE FIRST RAIN.",
            "IN KYOTO A CISTERN STAYS EMPTY.",
            "A TEMPLE BURNS WITH NO WATER NEAR.",
            "THE SUTRAS GO UP WITH THE ROOF.",
        ],
    ],
    // 5 formica
    [
        [
            "THE FOOT WAITS FOR THE ANT TO PASS.",
            "THE SEED REACHES THE NEST.",
            "A FIG SHADES A COURTYARD AT NALANDA.",
            "A MONK READS ONE PAGE MORE IN THE SHADE.",
            "A PROOF ABOUT ZERO SURVIVES THE RAINS.",
        ],
        [
            "THE FOOT CIRCLES THE ANT.",
            "THE ANT CHANGES ROAD AND DROPS THE SEED.",
            "A SEEDLING GROWS BESIDE A ROAD TO AXUM.",
            "CARAVANS REST THERE A CENTURY LATER.",
            "A MARKET KEEPS THE NAME OF A TREE.",
        ],
        [
            "THE ANT DIES UNDER A BLIND STEP.",
            "THE SEED NEVER SPROUTS.",
            "ON RAPA NUI ONE PALM FEWER TAKES ROOT.",
            "THE LAST CANOE WOOD RUNS OUT SOONER.",
            "AN ISLAND FORGETS THE SEA.",
        ],
    ],
    // 6 echo
    [
        [
            "A SHOUT GOES DOWN INTO THE VALLEY.",
            "A GOAT STARTLES AND DROPS A BELL.",
            "A SHEPHERD'S BOY FINDS A ROMAN COIN.",
            "THE COIN BUYS A PLACE IN A SCHOOL.",
            "A CODE OF LAWS OUTLASTS THE EMPIRE.",
        ],
        [
            "THE ECHO DIES UNANSWERED.",
            "A HARE KEEPS STILL IN THE GRASS.",
            "A HUNTER OF BAGHDAD COMES BACK EMPTY.",
            "THE EVENING GOES TO A PROBLEM INSTEAD.",
            "ALGEBRA GAINS ONE NIGHT.",
        ],
        [
            "THE ECHO COMES BACK CHANGED.",
            "A SECOND VOICE HIDES IN THE SOUND.",
            "AT TEOTIHUACAN A FLUTE FINDS A NEW TONE.",
            "THE TONE OPENS A FEAST EACH SPRING.",
            "A CITY KEEPS TIME BY A SOUND.",
        ],
    ],
    // 7 ventus
    [
        [
            "THE BODY LEANS INTO THE GUST.",
            "A SEED OF THISTLE FLIES PAST THE EAR.",
            "A THISTLE TAKES ROOT IN CÓRDOBA.",
            "A GARDENER LEARNS A GRAFT THAT YEAR.",
            "ORANGES REACH A COLD COUNTRY.",
        ],
        [
            "THE CLIMB WAITS FOR THE WIND TO PASS.",
            "A GULL RIDES THE GUST OUT TO SEA.",
            "A CANOE OF TONGA READS THE BIRD'S LINE.",
            "THE CANOE FINDS AN ISLAND BY THE GULL.",
            "A ROUTE IS SUNG FOR TWENTY GENERATIONS.",
        ],
        [
            "THE FACE TURNS FROM THE DUST.",
            "A MOTH SHELTERS IN THE COLLAR.",
            "A MOTH RIDES A BARGE BOUND FOR ANGKOR.",
            "A SILK HARVEST FAILS ONE SEASON.",
            "A TEMPLE LOSES A DECADE.",
        ],
    ],
];

/// One aphorism per clinamen, drawn at its branching node (REQ-L4-12).
pub const APHORISMI: [&str; 8] = [
    "□LEAF → ◊FIRE",
    "□STONE → ◊ROME",
    "□SHADOW → ◊SIGN",
    "□DROP → ◊COUNT",
    "□CRACK → ◊WORD",
    "□SEED → ◊TREE",
    "□ECHO → ◊LAW",
    "□WIND → ◊ROUTE",
];

/// The lines of Diogenes, Heraclitus and the daemon (`eventus` 13..=15).
pub const PERSONAE: [&str; 3] = [
    "WHY CARRY A STONE WITH NO OWNER?",
    "THE SAME STONE DOES NOT RISE TWICE.",
    "THIS CLIMB AGAIN, AND AGAIN, FOREVER?",
];

/// The two lines of the recognition (`eventus` 12): intimate, then abstract.
pub const AGNITIO: [&str; 2] = [
    "THE HEAD RESTS ON THE COLD STONE.",
    "THE CLIMB IS A DREAM. THE STONE IS NOT.",
];

/// The line of the metalepsis (`eventus` 11), abstract.
pub const METALEPSIS: &str = "THE LOOP RUNS THE HAND.";

/// The lines of the interruption: two epochal, then two civil.
pub const INTERRUPTIO: [&str; 4] = [
    "THE HAND MOVES. THE WAR WAITS.",
    "A CURSOR STOPS A CENTURY.",
    "A GATE OF TROY STAYS HALF SHUT.",
    "A CARAVAN CAMPS AT A SEALED WELL.",
];
