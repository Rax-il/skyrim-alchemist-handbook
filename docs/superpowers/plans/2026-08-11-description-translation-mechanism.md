# Механизм перевода описаний ингредиентов Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** дать `migrate_i18n()` возможность подставлять реальный перевод
описания вместо жёстко зашитой пустой строки на не-`ru` языках — и для 110
обычных ингредиентов (по одному тексту на ингредиент), и для 4 общих
описаний дополнений (Rare Curios/Рыбалка/Святые и соблазнители/Чума
мертвецов, по одному тексту на всю группу). Заодно закрыть единственный
известный пробел с переводом **имени** — «Смертная плоть» (плагин Plague
of the Dead не попал в скачанный архив официальных строк).

**Architecture:** новый файл `seed_description_translations.rs` — источник
переводов, отдельный от пайплайн-генерируемого `seed_translations.rs`
(разная природа данных: там — официальные строки Bethesda, здесь —
литературный перевод). Две задачи: Task 1 создаёт файл с реальным (но
небольшим — 40 строк) контентом и регистрирует модуль; Task 2 меняет
`migrate_i18n()`, чтобы он этот контент реально использовал. Само массовое
наполнение 110 обычных описаний (~880 строк) — отдельная, более поздняя
работа, в этот план не входит (см. Global Constraints).

**Tech Stack:** Rust/rusqlite (существующий стек, TDD как в
предыдущих Rust-планах B2a/B3+B4 — RED здесь означает "не компилируется
или падает ассерт под новую ожидаемую сигнатуру", GREEN — реализация
делает тест зелёным).

## Global Constraints

- **`DESCRIPTION_TRANSLATIONS` (110 обычных ингредиентов) начинается
  пустым** (`&[]`) — наполнение реальным переводом всех 110 текстов на 8
  языков — отдельная, более поздняя партиями работа, явно вне рамок этого
  плана (см. дизайн-документ, раздел "Порядок выполнения").
- **4 общих описания дополнений и 1 `NAME_OVERRIDES` — наполняются реальным
  переводом сразу**, целиком, в этом плане (40 коротких строк — нужны, чтобы
  осмысленно протестировать механизм, и это самостоятельно полезный контент,
  не заглушка).
- **Порядок языковых кодов — `de, en, es, fr, it, ja, pl, zh-Hant`**
  (алфавитный) — сверено с реальным порядком в `seed_translations.rs` для
  `"Абесинский окунь"`, не выдумано.
- **`seed_description_translations.rs` — НЕ автогенерируемый файл**, в
  отличие от соседа `seed_translations.rs` — редактируется вручную, шапка
  файла должна явно это говорить (перевод сделан Claude, не официальные
  строки игры).
- **Второй проход по общим описаниям в `migrate_i18n()` должен идти строго
  после основного цикла** — обновляемая строка `component_translations`
  создаётся основным циклом (через перевод имени); `UPDATE` по
  несуществующей строке — тихий no-op, не ошибка (то же самое уже
  происходит для «Смертная плоть» сегодня, пока `NAME_OVERRIDES` не создаст
  для неё нужные строки).
- **Качество перевода:** en/fr/de/it/es/pl — уверенность высокая; ja/zh-Hant
  — грамматически корректно, но без вычитки носителем (см. дизайн-документ)
  — это осознанно принятое ограничение, не повод блокировать план.

---

### Task 1: `src-tauri/src/seed_description_translations.rs` (новый файл)

**Files:**
- Create: `src-tauri/src/seed_description_translations.rs`
- Modify: `src-tauri/src/lib.rs:1-9` (регистрация модуля)

**Interfaces:**
- Produces (используется в Task 2, `db.rs`):
  - `pub const DESCRIPTION_TRANSLATIONS: &[(&str, &str, &str)]` — `&[]`
  - `pub const NAME_OVERRIDES: &[(&str, &str, &str)]` — 8 записей
  - `pub const RARE_CURIOS_DESCRIPTION_TRANSLATIONS: &[(&str, &str)]` — 8 записей
  - `pub const FISHING_DESCRIPTION_TRANSLATIONS: &[(&str, &str)]` — 8 записей
  - `pub const SAINTS_DESCRIPTION_TRANSLATIONS: &[(&str, &str)]` — 8 записей
  - `pub const PLAGUE_DESCRIPTION_TRANSLATIONS: &[(&str, &str)]` — 8 записей

- [ ] **Step 1: Создать файл**

```rust
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
```

- [ ] **Step 2: Зарегистрировать модуль в `lib.rs`**

Замените:

```rust
mod addons;
mod commands;
mod creations;
mod db;
mod layout;
mod paths;
mod rare_curios;
mod seed_data;
mod seed_translations;
```

на:

```rust
mod addons;
mod commands;
mod creations;
mod db;
mod layout;
mod paths;
mod rare_curios;
mod seed_data;
mod seed_description_translations;
mod seed_translations;
```

- [ ] **Step 3: Проверить компиляцию**

Run: `cd src-tauri && cargo check`
Expected: PASS — новый модуль пока никем не используется (кроме
объявления), но сам по себе должен компилироваться без ошибок и без
предупреждений о неиспользуемых константах (все они `pub`, значит `dead_code`
warning не сработает даже раньше, чем их подключат в Task 2).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/seed_description_translations.rs src-tauri/src/lib.rs
git commit -m "feat: add seed_description_translations.rs with group descriptions + name override (Task 1 of description-translation-mechanism)"
```

---

### Task 2: `migrate_i18n()` — использовать переводы описаний

**Files:**
- Modify: `src-tauri/src/db.rs`

**Interfaces:**
- Consumes: `DESCRIPTION_TRANSLATIONS`, `NAME_OVERRIDES`,
  `RARE_CURIOS_DESCRIPTION_TRANSLATIONS`, `FISHING_DESCRIPTION_TRANSLATIONS`,
  `SAINTS_DESCRIPTION_TRANSLATIONS`, `PLAGUE_DESCRIPTION_TRANSLATIONS` (Task 1).
  `RARE_CURIOS_INGREDIENTS` (`rare_curios.rs`, `&[(&str, [&str;4])]`),
  `FISHING_INGREDIENTS`/`SAINTS_INGREDIENTS`/`PLAGUE_INGREDIENTS`
  (`creations.rs`, `&[&str]`) — все уже импортированы в `db.rs`.

- [ ] **Step 1: Переписать/добавить тесты (RED)**

В `#[cfg(test)] mod rare_curios_tests`, сразу после теста
`migration_skips_name_with_no_official_translation` (уже существующий,
проверяет ровно "Смертная плоть" — после этого плана его поведение
меняется, см. ниже), добавьте:

```rust
    /// До этого плана "Смертная плоть" не имела перевода имени ни на одном
    /// не-ru языке (единственный пробел Plan A). NAME_OVERRIDES закрывает
    /// его напрямую (не через официальный пайплайн).
    #[test]
    fn migration_applies_name_overrides() {
        let db_path = temp_db_path("i18n_name_overrides");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let component_id = component_id_by_name(&conn, "Смертная плоть");
        let en_name: String = conn
            .query_row(
                "SELECT name FROM component_translations WHERE component_id = ?1 AND lang = 'en'",
                params![component_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(en_name, "Flesh of the Dead");

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    /// DESCRIPTION_TRANSLATIONS сейчас пуст (наполняется отдельно, партиями
    /// — см. design doc "Порядок выполнения"), значит для любого обычного
    /// ингредиента description на non-ru языке должен оставаться пустой
    /// строкой (не мусором, не паникой) — тот же принцип, что уже применён
    /// к именам без официального совпадения. Как только первая партия
    /// DESCRIPTION_TRANSLATIONS появится, этот тест стоит расширить
    /// проверкой конкретного переведённого текста.
    #[test]
    fn migration_defaults_missing_description_to_empty() {
        let db_path = temp_db_path("i18n_description_smoke");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();
        // Смоук-тест: миграция вообще не падает и не создаёт мусор в
        // description для непереведённого ru_name.
        let component_id = component_id_by_name(&conn, "Белянка");
        let en_description: String = conn
            .query_row(
                "SELECT description FROM component_translations WHERE component_id = ?1 AND lang = 'en'",
                params![component_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(en_description, "", "DESCRIPTION_TRANSLATIONS ещё пуст — должно быть пусто, не мусор");

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    /// Общее описание Rare Curios должно долетать до КАЖДОГО ингредиента
    /// группы на не-ru языке — не только до "первого" или дублироваться в
    /// исходных данных.
    #[test]
    fn migration_applies_rare_curios_group_description() {
        let db_path = temp_db_path("i18n_rare_curios_group_desc");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let component_id = component_id_by_name(&conn, "Шляпки гифоломы");
        let en_description: String = conn
            .query_row(
                "SELECT description FROM component_translations WHERE component_id = ?1 AND lang = 'en'",
                params![component_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            en_description.starts_with("A rare find from beyond Skyrim"),
            "получено: {en_description}"
        );

        // Второй ингредиент той же группы — тот же текст, не пусто.
        let component_id2 = component_id_by_name(&conn, "Гнилая чешуйка");
        let en_description2: String = conn
            .query_row(
                "SELECT description FROM component_translations WHERE component_id = ?1 AND lang = 'en'",
                params![component_id2],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(en_description2, en_description);

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    /// "Смертная плоть" не входила в TRANSLATIONS (нет официального
    /// совпадения) — до NAME_OVERRIDES у неё вообще не было non-ru строки
    /// component_translations, значит и общее описание Plague of the Dead
    /// не могло долететь (UPDATE по несуществующей строке — no-op). После
    /// этого плана NAME_OVERRIDES создаёт строку в основном цикле, и второй
    /// проход по PLAGUE_INGREDIENTS корректно её обновляет.
    #[test]
    fn migration_applies_plague_group_description_via_name_override() {
        let db_path = temp_db_path("i18n_plague_group_desc");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let component_id = component_id_by_name(&conn, "Смертная плоть");
        let en_description: String = conn
            .query_row(
                "SELECT description FROM component_translations WHERE component_id = ?1 AND lang = 'en'",
                params![component_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(
            en_description.starts_with("A sinister find from the Plague of the Dead add-on"),
            "получено: {en_description}"
        );

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }
```

- [ ] **Step 2: Убедиться, что не компилируется/не проходит (RED)**

Run: `cd src-tauri && cargo test --lib migration_applies_name_overrides`
Expected: FAIL — `en_name` вернёт ошибку (строка `component_translations`
для `lang='en'` у "Смертная плоть" ещё не существует, `query_row` вернёт
`QueryReturnedNoRows`), либо (после Step 1 остальных тестов) тесты
`migration_applies_rare_curios_group_description`/
`migration_applies_plague_group_description_via_name_override` упадут
аналогично — общего описания в `description` пока нет, там пустая строка.

- [ ] **Step 3: Импорты**

Замените:

```rust
use crate::rare_curios::{RARE_CURIOS_DESCRIPTION, RARE_CURIOS_INGREDIENTS, RARE_CURIOS_PROPERTIES};
use crate::seed_data::{DAWNGUARD_INGREDIENTS, DRAGONBORN_INGREDIENTS, HEARTHFIRE_INGREDIENTS, INGREDIENTS, MEDIA, PROPERTIES};
use crate::seed_translations::TRANSLATIONS;
```

на:

```rust
use crate::rare_curios::{RARE_CURIOS_DESCRIPTION, RARE_CURIOS_INGREDIENTS, RARE_CURIOS_PROPERTIES};
use crate::seed_data::{DAWNGUARD_INGREDIENTS, DRAGONBORN_INGREDIENTS, HEARTHFIRE_INGREDIENTS, INGREDIENTS, MEDIA, PROPERTIES};
use crate::seed_description_translations::{
    DESCRIPTION_TRANSLATIONS, FISHING_DESCRIPTION_TRANSLATIONS, NAME_OVERRIDES,
    PLAGUE_DESCRIPTION_TRANSLATIONS, RARE_CURIOS_DESCRIPTION_TRANSLATIONS,
    SAINTS_DESCRIPTION_TRANSLATIONS,
};
use crate::seed_translations::TRANSLATIONS;
```

- [ ] **Step 4: Основной цикл — подставлять реальное описание + учитывать `NAME_OVERRIDES`**

Замените:

```rust
    for (ru_name, lang, text) in TRANSLATIONS {
        if let Some(&id) = component_ids.get(*ru_name) {
            tx.execute(
                "INSERT INTO component_translations (component_id, lang, name, description)
                 VALUES (?1, ?2, ?3, '')",
                params![id, lang, text],
            )?;
            continue;
        }
        if let Some(&id) = property_ids.get(*ru_name) {
            tx.execute(
                "INSERT INTO property_translations (property_id, lang, name)
                 VALUES (?1, ?2, ?3)",
                params![id, lang, text],
            )?;
        }
    }

    tx.commit()?;
    Ok(())
}
```

на:

```rust
    let description_by_name_lang: HashMap<(&str, &str), &str> = DESCRIPTION_TRANSLATIONS
        .iter()
        .map(|(name, lang, desc)| ((*name, *lang), *desc))
        .collect();

    for (ru_name, lang, text) in TRANSLATIONS.iter().chain(NAME_OVERRIDES.iter()) {
        if let Some(&id) = component_ids.get(*ru_name) {
            let description = description_by_name_lang.get(&(*ru_name, *lang)).copied().unwrap_or("");
            tx.execute(
                "INSERT INTO component_translations (component_id, lang, name, description)
                 VALUES (?1, ?2, ?3, ?4)",
                params![id, lang, text, description],
            )?;
            continue;
        }
        if let Some(&id) = property_ids.get(*ru_name) {
            tx.execute(
                "INSERT INTO property_translations (property_id, lang, name)
                 VALUES (?1, ?2, ?3)",
                params![id, lang, text],
            )?;
        }
    }

    // Общие описания дополнений — второй проход, СТРОГО после основного
    // цикла: обновляемая строка (component_id, lang) должна уже
    // существовать (создана выше через перевод имени). UPDATE по
    // несуществующей строке — тихий no-op, не ошибка (например, для
    // ингредиента без перевода имени вообще).
    fn apply_group_description(
        tx: &rusqlite::Transaction,
        component_ids: &HashMap<String, i64>,
        names: &[&str],
        translations: &[(&str, &str)],
    ) -> rusqlite::Result<()> {
        for name in names {
            if let Some(&id) = component_ids.get(*name) {
                for (lang, description) in translations {
                    tx.execute(
                        "UPDATE component_translations SET description = ?1 WHERE component_id = ?2 AND lang = ?3",
                        params![description, id, lang],
                    )?;
                }
            }
        }
        Ok(())
    }

    let rare_curios_names: Vec<&str> = RARE_CURIOS_INGREDIENTS.iter().map(|(name, _)| *name).collect();
    apply_group_description(&tx, &component_ids, &rare_curios_names, RARE_CURIOS_DESCRIPTION_TRANSLATIONS)?;
    apply_group_description(&tx, &component_ids, FISHING_INGREDIENTS, FISHING_DESCRIPTION_TRANSLATIONS)?;
    apply_group_description(&tx, &component_ids, SAINTS_INGREDIENTS, SAINTS_DESCRIPTION_TRANSLATIONS)?;
    apply_group_description(&tx, &component_ids, PLAGUE_INGREDIENTS, PLAGUE_DESCRIPTION_TRANSLATIONS)?;

    tx.commit()?;
    Ok(())
}
```

- [ ] **Step 5: Убедиться, что компилируется и все тесты проходят**

Run: `cd src-tauri && cargo test --lib`
Expected: все тесты (27 = 23 текущих + 4 новых) — `ok`.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat: apply description translations + name overrides in migrate_i18n (Task 2 of description-translation-mechanism)"
```

---

## После выполнения плана

Механизм готов: любая запись в `DESCRIPTION_TRANSLATIONS` (110 обычных
ингредиентов) или в четырёх `..._DESCRIPTION_TRANSLATIONS` массивах
(дополнения) автоматически долетает до БД при следующем запуске — без
изменений в коде. `NAME_OVERRIDES` закрывает единственный пробел
официального сопоставления имён. Наполнение 110 обычных описаний реальным
переводом — следующая, отдельная от этого плана работа, партиями (см.
design doc, "Порядок выполнения") — просто добавление записей в
`DESCRIPTION_TRANSLATIONS`, без дальнейших изменений `db.rs`.
