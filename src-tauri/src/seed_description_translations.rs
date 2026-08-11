// seed_description_translations.rs — литературный перевод описаний
// ингредиентов на не-русские языки. В отличие от seed_translations.rs
// (официальные строки Bethesda, сгенерировано пайплайном tools/i18n,
// НЕ редактировать вручную) — этот файл содержит перевод, сделанный
// вручную (Claude), никакого отношения к официальным строкам игры не
// имеющий. Редактируется вручную, партиями — см.
// docs/superpowers/specs/2026-08-10-i18n-storage-design.md, дополнение
// "перевод описаний ингредиентов на 8 языков".

/// Переводы описаний 110 обычных ингредиентов (seed_data.rs::MEDIA) —
/// (ru_name, lang, описание). Наполняется постепенно, отдельными партиями
/// — см. design doc. Пока пусто: migrate_i18n() корректно обрабатывает
/// отсутствие записи (description = '', как и раньше).
pub const DESCRIPTION_TRANSLATIONS: &[(&str, &str, &str)] = &[];

/// Переводы имени для случаев, когда официального совпадения от Plan A не
/// нашлось — сейчас единственный случай: "Смертная плоть" (Plague of the
/// Dead), плагин отсутствовал в скачанном архиве официальных строк. Тот же
/// формат и та же роль в migrate_i18n(), что и TRANSLATIONS
/// (seed_translations.rs), просто источник — не официальный пайплайн.
pub const NAME_OVERRIDES: &[(&str, &str, &str)] = &[
    ("Смертная плоть", "de", "Fleisch der Toten"),
    ("Смертная плоть", "en", "Flesh of the Dead"),
    ("Смертная плоть", "es", "Carne de los muertos"),
    ("Смертная плоть", "fr", "Chair des morts"),
    ("Смертная плоть", "it", "Carne dei morti"),
    ("Смертная плоть", "ja", "死者の肉"),
    ("Смертная плоть", "pl", "Ciało zmarłego"),
    ("Смертная плоть", "zh-Hant", "死者之肉"),
];

/// Общее описание для всех 52 ингредиентов дополнения Rare Curios —
/// (lang, описание). Ru-оригинал: rare_curios.rs::RARE_CURIOS_DESCRIPTION.
pub const RARE_CURIOS_DESCRIPTION_TRANSLATIONS: &[(&str, &str)] = &[
    ("de", "Ein seltener Fund von jenseits Himmelsrands: Diese Zutat gelangt dank der Erweiterung Rare Curios in die Hände des Drachenbluts — Khajiit-Händler bringen solche Kuriositäten in ihren Karawanen aus fernen Landen: aus Cyrodiil, Morrowind und den Wogenden Inseln."),
    ("en", "A rare find from beyond Skyrim: this ingredient reaches the Dragonborn's hands thanks to the Rare Curios add-on — Khajiit merchants bring such curiosities in their caravans from distant lands: Cyrodiil, Morrowind, and the Shivering Isles."),
    ("es", "Un hallazgo poco común llegado de más allá de Skyrim: este ingrediente llega a manos del Sangre de Dragón gracias a la ampliación Rare Curios — los mercaderes Khajiitas traen estas rarezas en sus caravanas desde tierras lejanas: Cyrodiil, Morrowind y las Islas Trémulas."),
    ("fr", "Une trouvaille rare venue d'au-delà de Bordeciel : cet ingrédient parvient entre les mains du Dovahkiin grâce à l'extension Rare Curios — des marchands khajiits apportent ces curiosités dans leurs caravanes depuis des terres lointaines : Cyrodiil, Morrowind et les Îles de la Démence."),
    ("it", "Un raro reperto da oltre i confini di Skyrim: questo ingrediente giunge nelle mani del Sangue di Drago grazie al componente aggiuntivo Rare Curios — i mercanti Khajiit portano queste curiosità nelle loro carovane da terre lontane: Cyrodiil, Morrowind e le Isole Tremule."),
    ("ja", "スカイリムの外からもたらされた稀少な品——この素材は「レア・キュリオス」拡張のおかげでドラゴンボーンの手に渡る。カジート商人たちがオブリビオン、モロウウィンド、シバリング・アイルズなど遠い地から、隊商にこうした珍品を運んでくる。"),
    ("pl", "Rzadkie znalezisko spoza granic Skyrim: ten składnik trafia w ręce Smoczej Krwi dzięki dodatkowi Rare Curios — kupcy Khajiit przywożą takie osobliwości w swoich karawanach z dalekich krain: Cyrodiil, Morrowind i Drżących Wysp."),
    ("zh-Hant", "來自天際境外的稀有發現:多虧「稀奇珍玩」擴充內容,這件材料才得以落入龍裔之手——凱季特商人將這些奇珍異寶從奧不里維恩、莫羅溫德與顫慄島等遙遠之地,經由商隊運來。"),
];

/// Общее описание для всех ингредиентов творения "Рыбалка" — (lang, описание).
/// Ru-оригинал: creations.rs::FISHING_DESC.
pub const FISHING_DESCRIPTION_TRANSLATIONS: &[(&str, &str)] = &[
    ("de", "Ein Fang aus der Erweiterung „Angeln\": Dieser Bewohner von Himmelsrands Flüssen, Seen und Meeren beißt bei jedem Angler an, der bereit ist, stundenlang am Wasser auf einen Biss zu warten — und wird dafür nicht nur mit Beute, sondern auch mit alchemistischem Wert belohnt."),
    ("en", "A catch from the Fishing add-on: this denizen of Skyrim's rivers, lakes, and seas takes the hook of any angler willing to sit by the water for hours waiting for a bite — rewarding them with more than just a catch, but alchemical value as well."),
    ("es", "Una captura de la ampliación «Pesca»: este habitante de los ríos, lagos y mares de Skyrim muerde el anzuelo de todo pescador dispuesto a esperar horas junto al agua — recompensándolo no solo con una presa, sino también con valor alquímico."),
    ("fr", "Une prise de l'extension « Pêche » : cet habitant des rivières, des lacs et des mers de Bordeciel mord à l'hameçon du pêcheur prêt à patienter des heures au bord de l'eau — récompensant sa patience non seulement par une prise, mais aussi par une valeur alchimique."),
    ("it", "Una cattura del componente aggiuntivo «Pesca»: questo abitante dei fiumi, dei laghi e dei mari di Skyrim abbocca all'amo di ogni pescatore disposto a sedersi per ore in attesa di un abboccamento — ricompensandolo non solo con una preda, ma anche con un valore alchemico."),
    ("ja", "「釣り」拡張の釣果——スカイリムの川、湖、海に棲むこの生き物は、何時間も水辺に座って当たりを待つ辛抱強い釣り人の針にかかる。得られるのは獲物だけでなく、錬金術的な価値でもある。"),
    ("pl", "Połów z dodatku „Wędkarstwo\": ten mieszkaniec rzek, jezior i mórz Skyrim łapie się na haczyk każdego wędkarza gotowego siedzieć godzinami nad wodą w oczekiwaniu na branie — nagradzając go nie tylko zdobyczą, ale i wartością alchemiczną."),
    ("zh-Hant", "「釣魚」擴充內容的漁獲:這種棲息在天際河流、湖泊與海洋中的生物,會咬上願意在水邊耐心等候數小時的釣客的魚鉤——回報他的不僅是漁獲,還有煉金術上的價值。"),
];

/// Общее описание для всех ингредиентов творения "Святые и соблазнители" —
/// (lang, описание). Ru-оригинал: creations.rs::SAINTS_DESC.
pub const SAINTS_DESCRIPTION_TRANSLATIONS: &[(&str, &str)] = &[
    ("de", "Eine Kuriosität jenseits der Realität: Diese Zutat steht in Verbindung mit der Kreation „Heilige und Verführer\" — einer Geschichte über eine Reise in Sheogoraths seltsames, verdrehtes Reich, wo selbst Pflanzen und Insekten der Logik des Wahnsinnsprinzen gehorchen."),
    ("en", "A curiosity from beyond reality: this ingredient is tied to the Saints & Seducers creation — a tale of a journey into the strange, twisted realm of Sheogorath, where even plants and insects obey the logic of the Mad Prince."),
    ("es", "Una rareza más allá de la realidad: este ingrediente está ligado a la creación «Santos y Seductores» — una historia sobre un viaje al extraño y retorcido reino de Sheogorath, donde hasta las plantas y los insectos obedecen la lógica del Príncipe Loco."),
    ("fr", "Une curiosité venue d'ailleurs : cet ingrédient est lié à la création « Saints et Séducteurs » — une histoire de voyage dans le royaume étrange et tordu de Sheogorath, où même les plantes et les insectes obéissent à la logique du Prince Fou."),
    ("it", "Una curiosità che sfida la realtà: questo ingrediente è legato alla creazione «Santi e Seduttori» — una storia di viaggio nel regno strano e contorto di Sheogorath, dove persino le piante e gli insetti obbediscono alla logica del Principe Folle."),
    ("ja", "現実の外からやってきた珍品——この素材は「聖者と誘惑者」というクリエーションに関わりがある。狂気の公シェオゴラスの支配する、奇妙でねじれた領域への旅の物語であり、そこでは草木や虫さえも狂気の公の理屈に従う。"),
    ("pl", "Osobliwość spoza rzeczywistości: ten składnik związany jest z kreacją „Święci i Uwodziciele\" — opowieścią o podróży do dziwnego, wykrzywionego królestwa Sheogoratha, gdzie nawet rośliny i owady podlegają logice Szalonego Księcia."),
    ("zh-Hant", "超脫現實的奇珍:這件材料與創作內容「聖徒與誘惑者」有關——講述一段前往希歐格拉斯那奇異扭曲領地的旅程,在那裡即使是花草蟲豸,也遵循著這位瘋狂親王的邏輯。"),
];

/// Общее описание для ингредиента творения "Чума мертвецов" —
/// (lang, описание). Ru-оригинал: creations.rs::PLAGUE_DESC.
pub const PLAGUE_DESCRIPTION_TRANSLATIONS: &[(&str, &str)] = &[
    ("de", "Ein unheilvoller Fund aus der Erweiterung „Pest der Toten\": Diese Zutat steht mit der Untotenplage über Himmelsrand in Verbindung und trägt die Spur der dunklen, zerstörerischen Magie, die die Toten aus ihren Gräbern erhebt."),
    ("en", "A sinister find from the Plague of the Dead add-on: this ingredient is bound to the undead scourge upon Skyrim, bearing the mark of the dark, destructive magic that raises the dead from their graves."),
    ("es", "Un hallazgo siniestro de la ampliación «Plaga de los Muertos»: este ingrediente está ligado al azote de no-muertos que asoló Skyrim y conserva la marca de la magia oscura y destructiva que levanta a los muertos de sus tumbas."),
    ("fr", "Une trouvaille sinistre de l'extension « Peste des morts » : cet ingrédient est lié au fléau des morts-vivants qui s'est abattu sur Bordeciel, et porte la marque de la magie sombre et destructrice qui relève les morts de leurs tombes."),
    ("it", "Un sinistro reperto del componente aggiuntivo «Peste dei Morti»: questo ingrediente è legato al flagello dei non-morti abbattutosi su Skyrim e reca il segno della magia oscura e distruttrice che risveglia i morti dalle loro tombe."),
    ("ja", "「死者の疫病」拡張がもたらした不吉な品——この素材はスカイリムを襲ったアンデッドの災厄に関わり、死者を墓から蘇らせる暗く破壊的な魔法の痕跡を宿している。"),
    ("pl", "Złowieszcze znalezisko z dodatku „Zaraza Umarłych\": ten składnik związany jest z plagą nieumarłych, która nawiedziła Skyrim, i nosi na sobie piętno mrocznej, niszczycielskiej magii wskrzeszającej zmarłych z grobów."),
    ("zh-Hant", "來自「死者瘟疫」擴充內容的不祥發現:這件材料與席捲天際的不死禍害有關,承載著那將死者從墳墓中喚醒的黑暗毀滅魔法之印記。"),
];
