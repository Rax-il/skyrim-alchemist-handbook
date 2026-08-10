# Переход db.rs/commands.rs на id (План B2a) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** заменить строковое имя ингредиента/свойства на `id: i64` как
основной идентификатор во всём `src-tauri/src/db.rs` и
`src-tauri/src/commands.rs`, а функции, возвращающие текст для отображения,
сделать явно зависящими от языка (`lang: &str`), читая его из новых таблиц
`component_translations`/`property_translations` (план B1, уже смержен).

**Architecture:** три последовательные задачи внутри одного Rust-крейта,
каждая оставляет `cargo check`/`cargo test --lib` зелёными: Task 1 —
читающие функции (списки, поиск сочетаний), Task 2 — пишущие функции
(создание/правка/удаление компонента), Task 3 — `commands.rs` +
`lib.rs` (регистрация команд), которые зависят от финальных сигнатур обеих
предыдущих задач. Задачи не делятся мельче — функции внутри каждой задачи
делят общее внутреннее представление (например, поиск сочетаний хранит
свойства компонента как `HashSet<i64>`, и `load_property_types`,
`load_components_with_properties`, `find_combinations`, `find_pairs`,
`find_max_combinations` должны поменяться синхронно, иначе крейт не
скомпилируется в промежуточном состоянии).

**Tech Stack:** Rust/rusqlite (существующий стек проекта). Ни один
frontend-файл не трогается — после этого плана `npm run build`/`tsc`
временно не проходят (сигнатуры Tauri-команд разошлись с `src/lib/api.ts`).
Это ожидаемое промежуточное состояние одной рабочей сессии — чинится
следующим планом (B2b, фронтенд), не отдельная поставка.

## Global Constraints

- **Переходят на `id: i64`** (принимают идентификатор существующей записи
  вместо имени): `component_properties`, `component_properties_with_types`,
  `component_media`, `component_addon`, `set_component_media`,
  `delete_component`, `update_component_properties`.
- **Остаются на имени, но становятся языкозависимыми:**
  `insert_component(name, lang, prop_ids)` — у новой записи `id` ещё не
  существует; `component_exists(name, lang)` — проверка дубликата теперь в
  рамках конкретного языка, не глобально.
- **Списковые функции** (`load_properties`, `load_component_names`,
  `load_component_names_filtered`) возвращают `Vec<(id, переведённое имя)>`
  вместо `Vec<String>` — фронтенду нужен стабильный `id` как значение
  `<Select>`, при этом показывать перевод.
- **`find_combinations`/`find_pairs`/`find_max_combinations`** остаются
  функциями, возвращающими уже отформатированный текстовый результат (не
  переделываются в структурированный вывод) — добавляют параметр
  `lang: &str`. `find_combinations`'s `selected` (свойства для поиска)
  становится `Vec<i64>`. `CombinationResult.components` становится
  `Vec<i64>` (было `Vec<String>`) — фронтенду понадобится сопоставлять эти
  id со своим уже загруженным списком компонентов, а не с именами.
- **`rename_component` удаляется целиком** — из `db.rs`, `commands.rs` и
  регистрации в `lib.rs`. Не вызывается из UI с сессии 2026-08-07
  (переименование существующих компонентов отключено намеренно, см.
  историю инцидента с "Аронией" в `SESSION_NOTES.md`) — мёртвый код.
- **Границы с B3/B4:** `lang` — настоящий параметр везде в `db.rs`/
  `commands.rs` (никакого хардкода внутри Rust-кода) — реальное
  переключение языка ещё не подключено (это B3, отдельный план), но
  плюмбинг должен быть полностью готов принять любой `lang`. Пишущие
  функции (`insert_component`, `set_component_media`) продолжают
  дополнительно писать в legacy-колонки `components.name`/`description` —
  `migrate_i18n()` (план B1) уже синхронизирует `lang='ru'` с этими
  колонками при каждом запуске (upsert), так что чтение и запись остаются
  согласованными без дополнительной работы. Вопрос "куда писать правки при
  выбранном не-русском языке" не решается в этом плане — это B4.
- **TDD в статически типизированном языке:** в Rust "падающий тест" для
  смены сигнатуры функции — это ошибка компиляции (`cargo check` не
  проходит), а не упавшее runtime-утверждение. Каждая задача ниже сначала
  переписывает тесты под НОВЫЕ сигнатуры (это гарантированно не
  скомпилируется, пока реализация не обновлена — это и есть RED), затем
  пишет реализацию (GREEN — компилируется и `cargo test` проходит).
- Схема таблиц переводов (уже создана в B1, не меняется здесь):
  `component_translations(component_id, lang, name, description, PRIMARY
  KEY(component_id, lang))`, `property_translations(property_id, lang,
  name, PRIMARY KEY(property_id, lang))`.
- **Известная особенность окружения (не баг этого плана):** `pub fn run()`
  в `src-tauri/src/lib.rs` вызывает `tauri::generate_context!()` — этот
  proc-макрос разворачивается на этапе компиляции и требует, чтобы
  `frontendDist` (`../dist`, см. `tauri.conf.json`) физически существовал,
  иначе падает ДАЖЕ `cargo check`/`cargo test --lib` (не только сборка
  приложения). Если в рабочей копии ещё не было `npm install && npm run
  build`, сначала выполните это в корне проекта — иначе первая же команда
  `cargo check` этого плана упадёт с ошибкой про `frontendDist`, не
  связанной с самим рефакторингом.

---

### Task 1: Читающие функции — списки, детали компонента, поиск сочетаний

**Files:**
- Modify: `src-tauri/src/db.rs`

**Interfaces:**
- Consumes: таблицы `component_translations`/`property_translations` из
  плана B1 (уже в `master`).
- Produces (используется в Task 3, `commands.rs`):
  - `pub struct PropertyInfo { pub id: i64, pub name: String }`
  - `pub struct ComponentNameInfo { pub id: i64, pub name: String }`
  - `pub fn load_properties(conn: &Connection, lang: &str) -> rusqlite::Result<Vec<PropertyInfo>>`
  - `pub fn load_property_types(conn: &Connection) -> rusqlite::Result<HashMap<i64, String>>`
  - `pub fn load_component_names(conn: &Connection, lang: &str) -> rusqlite::Result<Vec<ComponentNameInfo>>`
  - `pub fn load_component_names_filtered(conn: &Connection, addons: &[Addon], lang: &str) -> rusqlite::Result<Vec<ComponentNameInfo>>`
  - `pub fn component_properties(conn: &Connection, component_id: i64, lang: &str) -> rusqlite::Result<Vec<String>>`
  - `pub struct PropWithType { pub id: i64, pub name: String, pub typ: String }`
  - `pub fn component_properties_with_types(conn: &Connection, component_id: i64, lang: &str) -> rusqlite::Result<Vec<PropWithType>>`
  - `pub fn component_media(conn: &Connection, component_id: i64, lang: &str) -> (Option<Vec<u8>>, String)`
  - `pub fn component_addon(conn: &Connection, component_id: i64) -> rusqlite::Result<Option<Addon>>`
  - `pub struct CombinationResult { pub components: Vec<i64>, pub line: String }`
  - `pub fn find_combinations(conn: &Connection, selected: &[i64], filter: &str, addons: &[Addon], lang: &str) -> rusqlite::Result<Vec<CombinationResult>>`
  - `pub fn find_pairs(conn: &Connection, filter: &str, addons: &[Addon], max_results: usize, lang: &str) -> rusqlite::Result<Vec<String>>`
  - `pub fn find_max_combinations(conn: &Connection, filter: &str, addons: &[Addon], max_results: usize, lang: &str) -> rusqlite::Result<Vec<String>>`
  - Test-only helpers (добавляются в `#[cfg(test)] mod rare_curios_tests`,
    используются и в Task 2): `fn component_id_by_name(conn: &Connection, name: &str) -> i64`,
    `fn property_id_by_name(conn: &Connection, name: &str) -> i64`.

- [ ] **Step 1: Добавить тестовые хелперы**

В `src-tauri/src/db.rs`, внутри `#[cfg(test)] mod rare_curios_tests`, сразу
после существующей `fn temp_db_path`, добавить:

```rust
    fn component_id_by_name(conn: &Connection, name: &str) -> i64 {
        conn.query_row("SELECT id FROM components WHERE name = ?1", params![name], |r| r.get(0))
            .unwrap()
    }

    fn property_id_by_name(conn: &Connection, name: &str) -> i64 {
        conn.query_row("SELECT id FROM properties WHERE name = ?1", params![name], |r| r.get(0))
            .unwrap()
    }
```

- [ ] **Step 2: Переписать существующие тесты под новые сигнатуры (RED — не скомпилируется)**

Замените ровно эти тесты в `#[cfg(test)] mod rare_curios_tests` на новые
версии (остальные тесты в модуле не трогать — они не вызывают ни одну из
меняющихся в этой задаче функций):

```rust
    #[test]
    fn migration_adds_rare_curios() {
        let db_path = temp_db_path("rare_curios");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let names = load_component_names(&conn, "ru").unwrap();
        assert!(names.iter().any(|c| c.name == "Шляпки гифоломы"), "Rare Curios компонент не добавлен");
        assert_eq!(names.len(), 110 + 52 + 18 - 2, "неверное общее число компонентов после миграции");

        let id = component_id_by_name(&conn, "Шляпки гифоломы");
        let props = component_properties(&conn, id, "ru").unwrap();
        let mut expected = vec![
            "Бешенство".to_string(),
            "Регенерация запаса сил".to_string(),
            "Урон здоровью".to_string(),
            "Уязвимость к яду".to_string(),
        ];
        expected.sort();
        let mut got = props.clone();
        got.sort();
        assert_eq!(got, expected);

        let (_, description) = component_media(&conn, id, "ru");
        assert!(description.contains("Rare Curios"), "описание не упоминает Rare Curios");

        drop(conn);
        let conn2 = ensure_db(&db_path).unwrap();
        let names2 = load_component_names(&conn2, "ru").unwrap();
        assert_eq!(names2.len(), 110 + 52 + 18 - 2, "повторная миграция задвоила компоненты");

        drop(conn2);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn creations_migration_adds_new_and_skips_duplicates() {
        let db_path = temp_db_path("creations");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();
        let names = load_component_names(&conn, "ru").unwrap();

        assert_eq!(names.len(), 110 + 52 + 18 - 2);
        assert!(names.iter().any(|c| c.name == "Смертная плоть"));
        assert!(names.iter().any(|c| c.name == "Стеклянный окунь"));

        let id = component_id_by_name(&conn, "Стеклянный окунь");
        let props = component_properties(&conn, id, "ru").unwrap();
        assert!(props.contains(&"Повышение искусства убеждать".to_string()));

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn migration_classifies_components_by_addon() {
        let db_path = temp_db_path("addon_classify");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        assert_eq!(component_addon(&conn, component_id_by_name(&conn, "Виноград джазби")).unwrap(), Some(Addon::BaseGame));
        assert_eq!(component_addon(&conn, component_id_by_name(&conn, "Жёлтый горноцвет")).unwrap(), Some(Addon::Dawnguard));
        assert_eq!(component_addon(&conn, component_id_by_name(&conn, "Вредозобник")).unwrap(), Some(Addon::Dragonborn));
        assert_eq!(component_addon(&conn, component_id_by_name(&conn, "Лососёвая икра")).unwrap(), Some(Addon::Hearthfire));
        assert_eq!(component_addon(&conn, component_id_by_name(&conn, "Шляпки гифоломы")).unwrap(), Some(Addon::RareCurios));
        assert_eq!(component_addon(&conn, component_id_by_name(&conn, "Золотая рыбка")).unwrap(), Some(Addon::Fishing));
        assert_eq!(component_addon(&conn, component_id_by_name(&conn, "Кричалка")).unwrap(), Some(Addon::SaintsAndSeducers));
        assert_eq!(component_addon(&conn, component_id_by_name(&conn, "Смертная плоть")).unwrap(), Some(Addon::PlagueOfTheDead));

        assert_eq!(component_addon(&conn, component_id_by_name(&conn, "Гнилая чешуйка")).unwrap(), Some(Addon::RareCurios));
        assert_eq!(component_addon(&conn, component_id_by_name(&conn, "Огненный гриб")).unwrap(), Some(Addon::RareCurios));

        drop(conn);
        let conn2 = ensure_db(&db_path).unwrap();
        assert_eq!(component_addon(&conn2, component_id_by_name(&conn2, "Гнилая чешуйка")).unwrap(), Some(Addon::RareCurios));

        drop(conn2);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn find_combinations_and_pairs_respect_addon_filter() {
        let db_path = temp_db_path("addon_filter_search");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let beshenstvo_id = property_id_by_name(&conn, "Бешенство");

        let empty_names = load_component_names_filtered(&conn, &[], "ru").unwrap();
        assert!(empty_names.is_empty());
        let empty_combos = find_combinations(&conn, &[beshenstvo_id], "", &[], "ru").unwrap();
        assert!(empty_combos.is_empty());
        let empty_pairs = find_pairs(&conn, "", &[], usize::MAX, "ru").unwrap();
        assert!(empty_pairs.is_empty());

        let only_rare_curios = [Addon::RareCurios];
        let names = load_component_names_filtered(&conn, &only_rare_curios, "ru").unwrap();
        assert_eq!(names.len(), 52, "в Rare Curios должно быть 52 компонента");
        assert!(names.iter().any(|c| c.name == "Шляпки гифоломы"));

        let combos = find_combinations(&conn, &[beshenstvo_id], "", &only_rare_curios, "ru").unwrap();
        assert!(!combos.is_empty(), "должны найтись сочетания внутри Rare Curios");
        for c in &combos {
            for &id in &c.components {
                assert_eq!(
                    component_addon(&conn, id).unwrap(),
                    Some(Addon::RareCurios),
                    "в результат просочился компонент не из Rare Curios: id={id}"
                );
            }
        }

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn find_max_combinations_finds_triples_with_at_least_four_effects() {
        let db_path = temp_db_path("max_combinations");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let all_addons = Addon::ALL;
        let lines = find_max_combinations(&conn, "", &all_addons, usize::MAX, "ru").unwrap();
        assert!(!lines.is_empty(), "должны найтись тройки с 4+ эффектами");

        let mut blocks: Vec<Vec<&str>> = vec![Vec::new()];
        for line in &lines {
            if line.is_empty() {
                blocks.push(Vec::new());
            } else {
                blocks.last_mut().unwrap().push(line.as_str());
            }
        }

        let mut effect_counts = Vec::new();
        for block in &blocks {
            assert!(
                block.len() >= 1 + MAX_COMBO_MIN_EFFECTS,
                "у тройки должно быть минимум 4 эффекта: {block:?}"
            );
            let header = block[0];
            assert_eq!(header.split(" + ").count(), 3, "заголовок должен перечислять ровно 3 ингредиента: {header}");
            let effects = &block[1..];
            assert!(effects.iter().all(|l| l.starts_with("    ")), "эффекты должны быть с отступом");
            effect_counts.push(effects.len());
        }

        let mut sorted = effect_counts.clone();
        sorted.sort_by(|a, b| b.cmp(a));
        assert_eq!(effect_counts, sorted);

        let prop_types = load_property_types(&conn).unwrap();
        let poison_lines = find_max_combinations(&conn, "Яд", &all_addons, usize::MAX, "ru").unwrap();
        assert!(!poison_lines.is_empty());
        for line in &poison_lines {
            if line.is_empty() || !line.starts_with("    ") {
                continue;
            }
            let prop_name = line.trim();
            let prop_id = property_id_by_name(&conn, prop_name);
            assert_eq!(prop_types.get(&prop_id).map(|s| s.as_str()), Some("Яд"));
        }

        let empty = find_max_combinations(&conn, "", &[], usize::MAX, "ru").unwrap();
        assert!(empty.is_empty());

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn find_pairs_and_max_combinations_respect_max_results() {
        let db_path = temp_db_path("max_results_limit");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        fn count_blocks(lines: &[String]) -> usize {
            if lines.is_empty() {
                return 0;
            }
            lines.iter().filter(|l| l.is_empty()).count() + 1
        }

        let all_addons = Addon::ALL;

        let unlimited_pairs = find_pairs(&conn, "", &all_addons, usize::MAX, "ru").unwrap();
        let total_pairs = count_blocks(&unlimited_pairs);
        assert!(total_pairs > 2, "для теста нужно больше пар, чем лимит");
        let limited_pairs = find_pairs(&conn, "", &all_addons, 2, "ru").unwrap();
        assert_eq!(count_blocks(&limited_pairs), 2);

        let unlimited_triples = find_max_combinations(&conn, "", &all_addons, usize::MAX, "ru").unwrap();
        let total_triples = count_blocks(&unlimited_triples);
        assert!(total_triples > 2, "для теста нужно больше троек, чем лимит");
        let limited_triples = find_max_combinations(&conn, "", &all_addons, 2, "ru").unwrap();
        assert_eq!(count_blocks(&limited_triples), 2);

        assert_eq!(limited_pairs, unlimited_pairs[..limited_pairs.len()]);
        assert_eq!(limited_triples, unlimited_triples[..limited_triples.len()]);

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn migration_populates_ru_translations_from_current_columns() {
        let db_path = temp_db_path("i18n_ru_backfill");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let component_id: i64 = conn
            .query_row("SELECT id FROM components WHERE name = ?1", params!["Белянка"], |r| r.get(0))
            .unwrap();
        let (name, description): (String, String) = conn
            .query_row(
                "SELECT name, description FROM component_translations WHERE component_id = ?1 AND lang = 'ru'",
                params![component_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(name, "Белянка");
        let (_, expected_description) = component_media(&conn, component_id, "ru");
        assert_eq!(description, expected_description);

        let property_id: i64 = conn
            .query_row("SELECT id FROM properties WHERE name = ?1", params!["Бешенство"], |r| r.get(0))
            .unwrap();
        let prop_name: String = conn
            .query_row(
                "SELECT name FROM property_translations WHERE property_id = ?1 AND lang = 'ru'",
                params![property_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(prop_name, "Бешенство");

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }
```

Не трогайте остальные тесты в модуле (`set_component_media_stores_and_clears_blob`,
`insert_component_is_tagged_user_added`, `migration_ru_upsert_reflects_user_edits`,
`migration_non_ru_rebuild_overwrites_stale_data`, `migration_purges_orphaned_translations_on_rowid_reuse`,
`migration_skips_name_with_no_official_translation`, `migration_is_idempotent`) —
их очередь в Task 2 (первые три) или они не вызывают ни одну из меняющихся
здесь функций (последние два).

- [ ] **Step 3: Убедиться, что не компилируется**

Run: `cd src-tauri && cargo check --tests`
Expected: множество ошибок компиляции — несовпадение количества аргументов
у `load_component_names`, `component_properties`, `component_media`,
`component_addon`, `find_combinations`, `find_pairs`, `find_max_combinations`,
`load_component_names_filtered`, `load_property_types` (используется как
`HashMap<String,_>` в старом коде, а тест уже ожидает `HashMap<i64,_>`).

- [ ] **Step 4: Реализовать — структуры и списковые функции**

Замените в `src-tauri/src/db.rs`:

```rust
pub fn load_properties(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM properties ORDER BY name")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    rows.collect()
}

pub fn load_property_types(conn: &Connection) -> rusqlite::Result<HashMap<String, String>> {
    let mut stmt = conn.prepare("SELECT name, type FROM properties")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
    let mut result = HashMap::new();
    for row in rows {
        let (name, typ) = row?;
        result.insert(name, typ);
    }
    Ok(result)
}

pub fn load_component_names(conn: &Connection) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM components ORDER BY name")?;
    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
    rows.collect()
}

pub fn load_component_names_filtered(conn: &Connection, addons: &[Addon]) -> rusqlite::Result<Vec<String>> {
    if addons.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = addon_placeholders(addons);
    let sql = format!("SELECT name FROM components WHERE addon IN ({placeholders}) ORDER BY name");
    let mut stmt = conn.prepare(&sql)?;
    let values: Vec<&str> = addons.iter().map(|a| a.as_str()).collect();
    let rows = stmt.query_map(rusqlite::params_from_iter(values), |r| r.get::<_, String>(0))?;
    rows.collect()
}
```

на:

```rust
#[derive(Serialize, Clone)]
pub struct PropertyInfo {
    pub id: i64,
    pub name: String,
}

#[derive(Serialize, Clone)]
pub struct ComponentNameInfo {
    pub id: i64,
    pub name: String,
}

pub fn load_properties(conn: &Connection, lang: &str) -> rusqlite::Result<Vec<PropertyInfo>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, pt.name
         FROM properties p
         JOIN property_translations pt ON pt.property_id = p.id AND pt.lang = ?1
         ORDER BY pt.name",
    )?;
    let rows = stmt.query_map(params![lang], |r| Ok(PropertyInfo { id: r.get(0)?, name: r.get(1)? }))?;
    rows.collect()
}

pub fn load_property_types(conn: &Connection) -> rusqlite::Result<HashMap<i64, String>> {
    let mut stmt = conn.prepare("SELECT id, type FROM properties")?;
    let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
    let mut result = HashMap::new();
    for row in rows {
        let (id, typ) = row?;
        result.insert(id, typ);
    }
    Ok(result)
}

/// id -> переведённое название свойства на lang. Приватная — используется
/// только внутри find_pairs/find_max_combinations для сборки строк
/// результата с отступом (эффекты сочетания).
fn load_property_names(conn: &Connection, lang: &str) -> rusqlite::Result<HashMap<i64, String>> {
    let mut stmt = conn.prepare("SELECT property_id, name FROM property_translations WHERE lang = ?1")?;
    let rows = stmt.query_map(params![lang], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
    let mut result = HashMap::new();
    for row in rows {
        let (id, name) = row?;
        result.insert(id, name);
    }
    Ok(result)
}

pub fn load_component_names(conn: &Connection, lang: &str) -> rusqlite::Result<Vec<ComponentNameInfo>> {
    let mut stmt = conn.prepare(
        "SELECT c.id, ct.name
         FROM components c
         JOIN component_translations ct ON ct.component_id = c.id AND ct.lang = ?1
         ORDER BY ct.name",
    )?;
    let rows = stmt.query_map(params![lang], |r| Ok(ComponentNameInfo { id: r.get(0)?, name: r.get(1)? }))?;
    rows.collect()
}

/// Список имён компонентов, ограниченный включёнными дополнениями (см.
/// SettingsModal.tsx на фронтенде) — используется выпадающим списком
/// "Компонент" в основном экране поиска. В отличие от load_component_names,
/// который остаётся без фильтра для "Редактировать базу" (там нужен полный
/// список независимо от текущих настроек фильтра).
pub fn load_component_names_filtered(conn: &Connection, addons: &[Addon], lang: &str) -> rusqlite::Result<Vec<ComponentNameInfo>> {
    if addons.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = addon_placeholders(addons);
    let sql = format!(
        "SELECT c.id, ct.name
         FROM components c
         JOIN component_translations ct ON ct.component_id = c.id AND ct.lang = ?1
         WHERE c.addon IN ({placeholders})
         ORDER BY ct.name"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut values: Vec<&str> = vec![lang];
    values.extend(addons.iter().map(|a| a.as_str()));
    let rows = stmt.query_map(rusqlite::params_from_iter(values), |r| {
        Ok(ComponentNameInfo { id: r.get(0)?, name: r.get(1)? })
    })?;
    rows.collect()
}
```

- [ ] **Step 5: Реализовать — детали компонента**

Замените:

```rust
pub fn component_properties(conn: &Connection, component_name: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT p.name
         FROM components c
         JOIN component_properties cp ON cp.component_id = c.id
         JOIN properties p ON p.id = cp.property_id
         WHERE c.name = ?1
         ORDER BY p.name",
    )?;
    let rows = stmt.query_map(params![component_name], |r| r.get::<_, String>(0))?;
    rows.collect()
}

#[derive(Serialize, Clone)]
pub struct PropWithType {
    pub name: String,
    pub typ: String,
}

pub fn component_properties_with_types(
    conn: &Connection,
    component_name: &str,
) -> rusqlite::Result<Vec<PropWithType>> {
    let mut stmt = conn.prepare(
        "SELECT p.name, p.type
         FROM components c
         JOIN component_properties cp ON cp.component_id = c.id
         JOIN properties p ON p.id = cp.property_id
         WHERE c.name = ?1
         ORDER BY p.name",
    )?;
    let rows = stmt.query_map(params![component_name], |r| {
        Ok(PropWithType {
            name: r.get(0)?,
            typ: r.get(1)?,
        })
    })?;
    rows.collect()
}

/// Возвращает (картинка как байты, если есть; описание). Картинка хранится
/// прямо в БД (колонка image, BLOB) — никаких внешних файлов/ссылок при
/// работе программы читать не нужно.
pub fn component_media(conn: &Connection, component_name: &str) -> (Option<Vec<u8>>, String) {
    conn.query_row(
        "SELECT image, description FROM components WHERE name = ?1",
        params![component_name],
        |r| Ok((r.get::<_, Option<Vec<u8>>>(0)?, r.get::<_, String>(1)?)),
    )
    .unwrap_or_default()
}

/// Дополнение-источник компонента (см. addons.rs) по имени.
pub fn component_addon(conn: &Connection, component_name: &str) -> rusqlite::Result<Option<Addon>> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT addon FROM components WHERE name = ?1",
            params![component_name],
            |r| r.get(0),
        )
        .optional()?;
    Ok(raw.and_then(|s| Addon::from_id(&s)))
}
```

на:

```rust
pub fn component_properties(conn: &Connection, component_id: i64, lang: &str) -> rusqlite::Result<Vec<String>> {
    let mut stmt = conn.prepare(
        "SELECT pt.name
         FROM component_properties cp
         JOIN property_translations pt ON pt.property_id = cp.property_id AND pt.lang = ?1
         WHERE cp.component_id = ?2
         ORDER BY pt.name",
    )?;
    let rows = stmt.query_map(params![lang, component_id], |r| r.get::<_, String>(0))?;
    rows.collect()
}

#[derive(Serialize, Clone)]
pub struct PropWithType {
    pub id: i64,
    pub name: String,
    pub typ: String,
}

pub fn component_properties_with_types(
    conn: &Connection,
    component_id: i64,
    lang: &str,
) -> rusqlite::Result<Vec<PropWithType>> {
    let mut stmt = conn.prepare(
        "SELECT p.id, pt.name, p.type
         FROM component_properties cp
         JOIN properties p ON p.id = cp.property_id
         JOIN property_translations pt ON pt.property_id = p.id AND pt.lang = ?1
         WHERE cp.component_id = ?2
         ORDER BY pt.name",
    )?;
    let rows = stmt.query_map(params![lang, component_id], |r| {
        Ok(PropWithType {
            id: r.get(0)?,
            name: r.get(1)?,
            typ: r.get(2)?,
        })
    })?;
    rows.collect()
}

/// Возвращает (картинка как байты, если есть; переведённое на lang
/// описание). Картинка не зависит от языка — хранится прямо в БД (колонка
/// image, BLOB), читается по id независимо от lang. Описание читается из
/// component_translations, а не из legacy-колонки components.description
/// напрямую — см. Global Constraints про согласованность с migrate_i18n.
pub fn component_media(conn: &Connection, component_id: i64, lang: &str) -> (Option<Vec<u8>>, String) {
    let image: Option<Vec<u8>> = conn
        .query_row("SELECT image FROM components WHERE id = ?1", params![component_id], |r| r.get(0))
        .unwrap_or(None);
    let description: String = conn
        .query_row(
            "SELECT description FROM component_translations WHERE component_id = ?1 AND lang = ?2",
            params![component_id, lang],
            |r| r.get(0),
        )
        .unwrap_or_default();
    (image, description)
}

/// Дополнение-источник компонента (см. addons.rs) по id. Не зависит от
/// языка.
pub fn component_addon(conn: &Connection, component_id: i64) -> rusqlite::Result<Option<Addon>> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT addon FROM components WHERE id = ?1",
            params![component_id],
            |r| r.get(0),
        )
        .optional()?;
    Ok(raw.and_then(|s| Addon::from_id(&s)))
}
```

- [ ] **Step 6: Реализовать — поиск сочетаний**

Замените приватную структуру и `load_components_with_properties`:

```rust
struct ComponentInfo {
    name: String,
    props: HashSet<String>,
}

fn load_components_with_properties(conn: &Connection, addons: &[Addon]) -> rusqlite::Result<Vec<ComponentInfo>> {
    if addons.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = addon_placeholders(addons);
    let sql = format!(
        "SELECT c.id, c.name, p.name
         FROM components c
         JOIN component_properties cp ON cp.component_id = c.id
         JOIN properties p ON p.id = cp.property_id
         WHERE c.addon IN ({placeholders})
         ORDER BY c.id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let values: Vec<&str> = addons.iter().map(|a| a.as_str()).collect();
    let mut rows = stmt.query(rusqlite::params_from_iter(values))?;

    let mut order: Vec<i64> = Vec::new();
    let mut by_id: HashMap<i64, ComponentInfo> = HashMap::new();
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let prop: String = row.get(2)?;
        let entry = by_id.entry(id).or_insert_with(|| {
            order.push(id);
            ComponentInfo {
                name: name.clone(),
                props: HashSet::new(),
            }
        });
        entry.props.insert(prop);
    }

    Ok(order.into_iter().map(|id| by_id.remove(&id).unwrap()).collect())
}
```

на:

```rust
struct SearchComponent {
    id: i64,
    name: String,
    props: HashSet<i64>,
}

/// addons — ингредиенты каких дополнений включать (см. Addon в addons.rs);
/// пустой список означает "ничего не включено" (осознанное состояние,
/// когда пользователь снял в Настройках все галочки), а не "фильтр не
/// задан" — поэтому сразу возвращаем пустой результат, не обращаясь к БД.
/// props — id свойств (не имена): сопоставление сочетаний должно работать
/// независимо от языка, а имя — только для отображения.
fn load_components_with_properties(conn: &Connection, addons: &[Addon], lang: &str) -> rusqlite::Result<Vec<SearchComponent>> {
    if addons.is_empty() {
        return Ok(Vec::new());
    }
    let placeholders = addon_placeholders(addons);
    let sql = format!(
        "SELECT c.id, ct.name, cp.property_id
         FROM components c
         JOIN component_translations ct ON ct.component_id = c.id AND ct.lang = ?1
         JOIN component_properties cp ON cp.component_id = c.id
         WHERE c.addon IN ({placeholders})
         ORDER BY c.id"
    );
    let mut stmt = conn.prepare(&sql)?;
    let mut values: Vec<&str> = vec![lang];
    values.extend(addons.iter().map(|a| a.as_str()));
    let mut rows = stmt.query(rusqlite::params_from_iter(values))?;

    let mut order: Vec<i64> = Vec::new();
    let mut by_id: HashMap<i64, SearchComponent> = HashMap::new();
    while let Some(row) = rows.next()? {
        let id: i64 = row.get(0)?;
        let name: String = row.get(1)?;
        let prop_id: i64 = row.get(2)?;
        let entry = by_id.entry(id).or_insert_with(|| {
            order.push(id);
            SearchComponent { id, name: name.clone(), props: HashSet::new() }
        });
        entry.props.insert(prop_id);
    }

    Ok(order.into_iter().map(|id| by_id.remove(&id).unwrap()).collect())
}
```

Замените `CombinationResult` и `find_combinations`:

```rust
/// Одна найденная смесь: компоненты (для фильтрации) и готовая строка.
#[derive(Serialize, Clone)]
pub struct CombinationResult {
    pub components: Vec<String>,
    pub line: String,
}

/// Ищет смеси из 2 или 3 компонентов, где каждое выбранное свойство
/// встречается минимум у двух компонентов смеси.
pub fn find_combinations(
    conn: &Connection,
    selected: &[String],
    filter: &str,
    addons: &[Addon],
) -> rusqlite::Result<Vec<CombinationResult>> {
    if selected.is_empty() {
        return Ok(Vec::new());
    }

    let components = load_components_with_properties(conn, addons)?;
    let prop_types = load_property_types(conn)?;
    let n = components.len();

    let satisfies_selected = |group: &[usize]| -> bool {
        for prop in selected {
            let count = group.iter().filter(|&&idx| components[idx].props.contains(prop)).count();
            if count < 2 {
                return false;
            }
        }
        true
    };

    let matches_filter = |group: &[usize]| -> bool {
        if filter.is_empty() {
            return true;
        }
        let mut counts: HashMap<&str, i32> = HashMap::new();
        for &idx in group {
            for prop in &components[idx].props {
                *counts.entry(prop.as_str()).or_insert(0) += 1;
            }
        }
        for (prop, count) in &counts {
            if *count < 2 {
                continue;
            }
            if prop_types.get(*prop).map(|s| s.as_str()) != Some(filter) {
                return false;
            }
        }
        true
    };

    let is_minimal = |group: &[usize]| -> bool {
        for skip in 0..group.len() {
            let sub: Vec<usize> = group
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != skip)
                .map(|(_, &v)| v)
                .collect();
            if satisfies_selected(&sub) {
                return false;
            }
        }
        true
    };

    let describe = |group: &[usize]| -> CombinationResult {
        let names: Vec<String> = group.iter().map(|&idx| components[idx].name.clone()).collect();
        let line = names.join(" + ");
        CombinationResult { components: names, line }
    };

    let mut results = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let group = [i, j];
            if satisfies_selected(&group) && matches_filter(&group) {
                results.push(describe(&group));
            }
        }
    }

    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                let group = [i, j, k];
                if satisfies_selected(&group) && is_minimal(&group) && matches_filter(&group) {
                    results.push(describe(&group));
                }
            }
        }
    }

    results.sort_by(|a, b| a.line.cmp(&b.line));
    Ok(results)
}
```

на:

```rust
/// Одна найденная смесь: id компонентов (для фильтрации на фронтенде по
/// enabledComponents) и готовая переведённая строка для отображения.
#[derive(Serialize, Clone)]
pub struct CombinationResult {
    pub components: Vec<i64>,
    pub line: String,
}

/// Ищет смеси из 2 или 3 компонентов, где каждое выбранное свойство
/// встречается минимум у двух компонентов смеси. selected — id свойств.
pub fn find_combinations(
    conn: &Connection,
    selected: &[i64],
    filter: &str,
    addons: &[Addon],
    lang: &str,
) -> rusqlite::Result<Vec<CombinationResult>> {
    if selected.is_empty() {
        return Ok(Vec::new());
    }

    let components = load_components_with_properties(conn, addons, lang)?;
    let prop_types = load_property_types(conn)?;
    let n = components.len();

    let satisfies_selected = |group: &[usize]| -> bool {
        for prop_id in selected {
            let count = group.iter().filter(|&&idx| components[idx].props.contains(prop_id)).count();
            if count < 2 {
                return false;
            }
        }
        true
    };

    let matches_filter = |group: &[usize]| -> bool {
        if filter.is_empty() {
            return true;
        }
        let mut counts: HashMap<i64, i32> = HashMap::new();
        for &idx in group {
            for &prop_id in &components[idx].props {
                *counts.entry(prop_id).or_insert(0) += 1;
            }
        }
        for (prop_id, count) in &counts {
            if *count < 2 {
                continue;
            }
            if prop_types.get(prop_id).map(|s| s.as_str()) != Some(filter) {
                return false;
            }
        }
        true
    };

    let is_minimal = |group: &[usize]| -> bool {
        for skip in 0..group.len() {
            let sub: Vec<usize> = group
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != skip)
                .map(|(_, &v)| v)
                .collect();
            if satisfies_selected(&sub) {
                return false;
            }
        }
        true
    };

    let describe = |group: &[usize]| -> CombinationResult {
        let ids: Vec<i64> = group.iter().map(|&idx| components[idx].id).collect();
        let names: Vec<&str> = group.iter().map(|&idx| components[idx].name.as_str()).collect();
        let line = names.join(" + ");
        CombinationResult { components: ids, line }
    };

    let mut results = Vec::new();

    for i in 0..n {
        for j in (i + 1)..n {
            let group = [i, j];
            if satisfies_selected(&group) && matches_filter(&group) {
                results.push(describe(&group));
            }
        }
    }

    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                let group = [i, j, k];
                if satisfies_selected(&group) && is_minimal(&group) && matches_filter(&group) {
                    results.push(describe(&group));
                }
            }
        }
    }

    results.sort_by(|a, b| a.line.cmp(&b.line));
    Ok(results)
}
```

Замените `find_pairs`:

```rust
/// Ищет пары компонентов, у которых совпадает минимум 2 свойства.
/// max_results — ограничение количества пар в результате (после сортировки
/// по убыванию числа общих свойств), см. поле "Макс. кол-во сочетаний" в
/// Настройках.
pub fn find_pairs(conn: &Connection, filter: &str, addons: &[Addon], max_results: usize) -> rusqlite::Result<Vec<String>> {
    let components = load_components_with_properties(conn, addons)?;
    let prop_types = load_property_types(conn)?;
    let n = components.len();

    struct PairResult {
        a: String,
        b: String,
        common: Vec<String>,
    }

    let mut pairs = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let mut common: Vec<String> = components[i]
                .props
                .iter()
                .filter(|p| components[j].props.contains(*p))
                .cloned()
                .collect();
            if common.len() < 2 {
                continue;
            }
            if !filter.is_empty() {
                let ok = common
                    .iter()
                    .all(|p| prop_types.get(p).map(|s| s.as_str()) == Some(filter));
                if !ok {
                    continue;
                }
            }
            common.sort();
            pairs.push(PairResult {
                a: components[i].name.clone(),
                b: components[j].name.clone(),
                common,
            });
        }
    }

    pairs.sort_by(|a, b| {
        b.common
            .len()
            .cmp(&a.common.len())
            .then_with(|| a.a.cmp(&b.a))
            .then_with(|| a.b.cmp(&b.b))
    });
    pairs.truncate(max_results);

    let mut lines = Vec::new();
    for (idx, p) in pairs.iter().enumerate() {
        if idx > 0 {
            lines.push(String::new());
        }
        lines.push(format!("{} + {}", p.a, p.b));
        for prop in &p.common {
            lines.push(format!("    {}", prop));
        }
    }
    Ok(lines)
}
```

на:

```rust
/// Ищет пары компонентов, у которых совпадает минимум 2 свойства.
/// max_results — ограничение количества пар в результате (после сортировки
/// по убыванию числа общих свойств), см. поле "Макс. кол-во сочетаний" в
/// Настройках.
pub fn find_pairs(conn: &Connection, filter: &str, addons: &[Addon], max_results: usize, lang: &str) -> rusqlite::Result<Vec<String>> {
    let components = load_components_with_properties(conn, addons, lang)?;
    let prop_types = load_property_types(conn)?;
    let prop_names = load_property_names(conn, lang)?;
    let n = components.len();

    struct PairResult {
        a: String,
        b: String,
        common: Vec<i64>,
    }

    let mut pairs = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            let mut common: Vec<i64> = components[i]
                .props
                .iter()
                .filter(|p| components[j].props.contains(*p))
                .cloned()
                .collect();
            if common.len() < 2 {
                continue;
            }
            if !filter.is_empty() {
                let ok = common
                    .iter()
                    .all(|p| prop_types.get(p).map(|s| s.as_str()) == Some(filter));
                if !ok {
                    continue;
                }
            }
            common.sort_by(|a, b| {
                let na = prop_names.get(a).map(|s| s.as_str()).unwrap_or("");
                let nb = prop_names.get(b).map(|s| s.as_str()).unwrap_or("");
                na.cmp(nb)
            });
            pairs.push(PairResult {
                a: components[i].name.clone(),
                b: components[j].name.clone(),
                common,
            });
        }
    }

    pairs.sort_by(|a, b| {
        b.common
            .len()
            .cmp(&a.common.len())
            .then_with(|| a.a.cmp(&b.a))
            .then_with(|| a.b.cmp(&b.b))
    });
    pairs.truncate(max_results);

    let mut lines = Vec::new();
    for (idx, p) in pairs.iter().enumerate() {
        if idx > 0 {
            lines.push(String::new());
        }
        lines.push(format!("{} + {}", p.a, p.b));
        for prop_id in &p.common {
            let name = prop_names.get(prop_id).map(|s| s.as_str()).unwrap_or("?");
            lines.push(format!("    {}", name));
        }
    }
    Ok(lines)
}
```

Замените `find_max_combinations`:

```rust
pub fn find_max_combinations(
    conn: &Connection,
    filter: &str,
    addons: &[Addon],
    max_results: usize,
) -> rusqlite::Result<Vec<String>> {
    let components = load_components_with_properties(conn, addons)?;
    let prop_types = load_property_types(conn)?;
    let n = components.len();

    struct TripleResult {
        names: [String; 3],
        effects: Vec<String>,
    }

    let mut triples = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                let group = [i, j, k];
                let mut counts: HashMap<&str, i32> = HashMap::new();
                for &idx in &group {
                    for prop in &components[idx].props {
                        *counts.entry(prop.as_str()).or_insert(0) += 1;
                    }
                }
                let mut effects: Vec<String> =
                    counts.into_iter().filter(|&(_, count)| count >= 2).map(|(p, _)| p.to_string()).collect();
                if effects.len() < MAX_COMBO_MIN_EFFECTS {
                    continue;
                }
                if !filter.is_empty() {
                    let ok = effects
                        .iter()
                        .all(|p| prop_types.get(p).map(|s| s.as_str()) == Some(filter));
                    if !ok {
                        continue;
                    }
                }
                effects.sort();
                triples.push(TripleResult {
                    names: [
                        components[i].name.clone(),
                        components[j].name.clone(),
                        components[k].name.clone(),
                    ],
                    effects,
                });
            }
        }
    }

    triples.sort_by(|a, b| b.effects.len().cmp(&a.effects.len()).then_with(|| a.names.cmp(&b.names)));
    triples.truncate(max_results);

    let mut lines = Vec::new();
    for (idx, t) in triples.iter().enumerate() {
        if idx > 0 {
            lines.push(String::new());
        }
        lines.push(t.names.join(" + "));
        for effect in &t.effects {
            lines.push(format!("    {}", effect));
        }
    }
    Ok(lines)
}
```

на:

```rust
pub fn find_max_combinations(
    conn: &Connection,
    filter: &str,
    addons: &[Addon],
    max_results: usize,
    lang: &str,
) -> rusqlite::Result<Vec<String>> {
    let components = load_components_with_properties(conn, addons, lang)?;
    let prop_types = load_property_types(conn)?;
    let prop_names = load_property_names(conn, lang)?;
    let n = components.len();

    struct TripleResult {
        names: [String; 3],
        effects: Vec<i64>,
    }

    let mut triples = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                let group = [i, j, k];
                let mut counts: HashMap<i64, i32> = HashMap::new();
                for &idx in &group {
                    for &prop_id in &components[idx].props {
                        *counts.entry(prop_id).or_insert(0) += 1;
                    }
                }
                let mut effects: Vec<i64> =
                    counts.into_iter().filter(|&(_, count)| count >= 2).map(|(p, _)| p).collect();
                if effects.len() < MAX_COMBO_MIN_EFFECTS {
                    continue;
                }
                if !filter.is_empty() {
                    let ok = effects
                        .iter()
                        .all(|p| prop_types.get(p).map(|s| s.as_str()) == Some(filter));
                    if !ok {
                        continue;
                    }
                }
                effects.sort_by(|a, b| {
                    let na = prop_names.get(a).map(|s| s.as_str()).unwrap_or("");
                    let nb = prop_names.get(b).map(|s| s.as_str()).unwrap_or("");
                    na.cmp(nb)
                });
                triples.push(TripleResult {
                    names: [
                        components[i].name.clone(),
                        components[j].name.clone(),
                        components[k].name.clone(),
                    ],
                    effects,
                });
            }
        }
    }

    triples.sort_by(|a, b| b.effects.len().cmp(&a.effects.len()).then_with(|| a.names.cmp(&b.names)));
    triples.truncate(max_results);

    let mut lines = Vec::new();
    for (idx, t) in triples.iter().enumerate() {
        if idx > 0 {
            lines.push(String::new());
        }
        lines.push(t.names.join(" + "));
        for prop_id in &t.effects {
            let name = prop_names.get(prop_id).map(|s| s.as_str()).unwrap_or("?");
            lines.push(format!("    {}", name));
        }
    }
    Ok(lines)
}
```

- [ ] **Step 7: Убедиться, что компилируется и тесты проходят**

Run: `cd src-tauri && cargo test --lib`
Expected: все тесты (старые 15 + без изменений в количестве, так как в этой
задаче тесты только переписаны, не добавлены) — `ok`. Компиляция без
предупреждений о неиспользуемых импортах/переменных.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat: switch db.rs read functions to id + lang (Task 1 of B2a)"
```

---

### Task 2: Пишущие функции — создание, правка, удаление компонента

**Files:**
- Modify: `src-tauri/src/db.rs`

**Interfaces:**
- Consumes: `component_id_by_name`/`property_id_by_name` (тестовые хелперы
  из Task 1), `component_addon`/`component_media`/`component_properties`
  (уже id-based после Task 1).
- Produces (используется в Task 3, `commands.rs`):
  - `pub fn component_exists(conn: &Connection, name: &str, lang: &str) -> rusqlite::Result<bool>`
  - `pub fn insert_component(conn: &Connection, name: &str, lang: &str, prop_ids: &[i64; 4]) -> rusqlite::Result<i64>`
    (возвращает id новой записи)
  - `pub fn delete_component(conn: &Connection, id: i64) -> rusqlite::Result<()>`
  - `pub fn set_component_media(conn: &Connection, id: i64, image_bytes: Option<&[u8]>, description: &str) -> rusqlite::Result<()>`
  - `pub fn update_component_properties(conn: &Connection, component_id: i64, prop_ids: &[i64; 4]) -> Result<(), String>`
  - `rename_component` — удаляется, никакой новой сигнатуры не производит.

- [ ] **Step 1: Переписать существующие тесты под новые сигнатуры (RED — не скомпилируется)**

Замените ровно эти тесты:

```rust
    #[test]
    fn set_component_media_stores_and_clears_blob() {
        let db_path = temp_db_path("set_media");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();
        let id = component_id_by_name(&conn, "Шляпки гифоломы");

        let bytes = vec![1u8, 2, 3, 4, 5];
        set_component_media(&conn, id, Some(&bytes), "новое описание").unwrap();
        let (image, description) = component_media(&conn, id, "ru");
        assert_eq!(image, Some(bytes));
        assert_eq!(description, "новое описание");

        set_component_media(&conn, id, None, "новое описание").unwrap();
        let (image2, _) = component_media(&conn, id, "ru");
        assert_eq!(image2, None);

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn insert_component_is_tagged_user_added() {
        let db_path = temp_db_path("addon_user_added");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let props = [
            property_id_by_name(&conn, "Бешенство"),
            property_id_by_name(&conn, "Замедление"),
            property_id_by_name(&conn, "Страх"),
            property_id_by_name(&conn, "Паралич"),
        ];
        let new_id = insert_component(&conn, "Тестовый ингредиент", "ru", &props).unwrap();
        assert_eq!(component_addon(&conn, new_id).unwrap(), Some(Addon::UserAdded));

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn migration_ru_upsert_reflects_user_edits() {
        let db_path = temp_db_path("i18n_ru_upsert");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let component_id = component_id_by_name(&conn, "Белянка");
        let (_, old_description) = component_media(&conn, component_id, "ru");

        set_component_media(&conn, component_id, None, "новое описание после правки пользователя").unwrap();

        drop(conn);
        let conn2 = ensure_db(&db_path).unwrap();

        let description: String = conn2
            .query_row(
                "SELECT description FROM component_translations WHERE component_id = ?1 AND lang = 'ru'",
                params![component_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(description, "новое описание после правки пользователя");
        assert_ne!(description, old_description);

        drop(conn2);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn migration_purges_orphaned_translations_on_rowid_reuse() {
        let db_path = temp_db_path("i18n_rowid_reuse");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let max_id: i64 = conn.query_row("SELECT MAX(id) FROM components", [], |r| r.get(0)).unwrap();
        let old_name: String = conn
            .query_row("SELECT name FROM components WHERE id = ?1", params![max_id], |r| r.get(0))
            .unwrap();

        let old_translation_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM component_translations WHERE component_id = ?1",
                params![max_id],
                |r| r.get(0),
            )
            .unwrap();
        assert!(old_translation_count > 0, "у удаляемого компонента должны быть переводы до удаления");

        delete_component(&conn, max_id).unwrap();

        // delete_component теперь сама чистит component_translations сразу,
        // без ожидания следующего перезапуска (см. её комментарий) —
        // проверяем это в той же сессии, до какой-либо вставки.
        let count_right_after_delete: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM component_translations WHERE component_id = ?1",
                params![max_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count_right_after_delete, 0, "delete_component должна сразу чистить переводы удалённого id");

        // SQLite переиспользует rowid максимального удалённого id для
        // следующей вставки без явного значения id — новый компонент
        // получит тот же max_id, что был у удалённого.
        let props = [
            property_id_by_name(&conn, "Бешенство"),
            property_id_by_name(&conn, "Замедление"),
            property_id_by_name(&conn, "Страх"),
            property_id_by_name(&conn, "Паралич"),
        ];
        let new_id = insert_component(&conn, "Новый компонент вместо удалённого", "ru", &props).unwrap();
        assert_eq!(new_id, max_id, "тест предполагает переиспользование rowid — иначе проверка ничего не доказывает");

        let new_translation_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM component_translations WHERE component_id = ?1",
                params![new_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(new_translation_count, 1, "у нового компонента должен быть ровно один (ru) перевод, без следов старого");

        drop(conn);
        let conn2 = ensure_db(&db_path).unwrap();

        let names_at_id: Vec<String> = {
            let mut stmt = conn2
                .prepare("SELECT name FROM component_translations WHERE component_id = ?1")
                .unwrap();
            let rows = stmt.query_map(params![new_id], |r| r.get::<_, String>(0)).unwrap();
            rows.collect::<Result<_, _>>().unwrap()
        };
        assert!(
            !names_at_id.iter().any(|n| n == &old_name),
            "перевод удалённого компонента просочился в новый с переиспользованным id: {names_at_id:?}"
        );
        assert!(
            names_at_id.iter().any(|n| n == "Новый компонент вместо удалённого"),
            "у нового компонента должен появиться свой ru-перевод: {names_at_id:?}"
        );

        drop(conn2);
        let _ = std::fs::remove_file(&db_path);
    }
```

Добавьте два новых теста (для функций без пре-существующего покрытия):

```rust
    #[test]
    fn update_component_properties_replaces_existing_links() {
        let db_path = temp_db_path("update_props");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let props = [
            property_id_by_name(&conn, "Бешенство"),
            property_id_by_name(&conn, "Замедление"),
            property_id_by_name(&conn, "Страх"),
            property_id_by_name(&conn, "Паралич"),
        ];
        let id = insert_component(&conn, "Тестовый ингредиент 2", "ru", &props).unwrap();

        let new_props = [
            property_id_by_name(&conn, "Водное дыхание"),
            property_id_by_name(&conn, "Невидимость"),
            property_id_by_name(&conn, "Урон здоровью"),
            property_id_by_name(&conn, "Паралич"),
        ];
        update_component_properties(&conn, id, &new_props).unwrap();

        let mut got = component_properties(&conn, id, "ru").unwrap();
        got.sort();
        let mut expected: Vec<String> = vec![
            "Водное дыхание".to_string(),
            "Невидимость".to_string(),
            "Паралич".to_string(),
            "Урон здоровью".to_string(),
        ];
        expected.sort();
        assert_eq!(got, expected);

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn component_exists_is_scoped_by_language() {
        let db_path = temp_db_path("exists_scoped");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        assert!(component_exists(&conn, "Белянка", "ru").unwrap());
        assert!(!component_exists(&conn, "Белянка", "en").unwrap(), "en-перевода с таким именем ещё нет");
        assert!(!component_exists(&conn, "Совершенно новое имя", "ru").unwrap());

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }
```

- [ ] **Step 2: Убедиться, что не компилируется**

Run: `cd src-tauri && cargo check --tests`
Expected: ошибки компиляции — несовпадение аргументов у `set_component_media`,
`insert_component`, `delete_component`, `update_component_properties`;
`component_exists` не найден с двумя аргументами.

- [ ] **Step 3: Реализовать**

Замените:

```rust
/// Сохраняет картинку (байты выбранного пользователем файла — либо None,
/// чтобы убрать картинку) и описание компонента.
pub fn set_component_media(
    conn: &Connection,
    name: &str,
    image_bytes: Option<&[u8]>,
    description: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE components SET image = ?1, description = ?2 WHERE name = ?3",
        params![image_bytes, description, name],
    )?;
    Ok(())
}

pub fn component_exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    let id: Option<i64> = conn
        .query_row("SELECT id FROM components WHERE name = ?1", params![name], |r| r.get(0))
        .optional()?;
    Ok(id.is_some())
}

/// Новый компонент, добавленный через "Редактировать базу", всегда
/// помечается Addon::UserAdded явно (не полагаемся на DEFAULT колонки) —
/// это единственный путь, которым в базе вообще может появиться такая
/// пометка (см. addons.rs).
pub fn insert_component(conn: &Connection, name: &str, prop_names: &[String; 4]) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO components (name, addon) VALUES (?1, ?2)",
        params![name, Addon::UserAdded.as_str()],
    )?;
    let component_id = conn.last_insert_rowid();
    for prop_name in prop_names {
        let prop_name = prop_name.trim();
        let property_id: i64 = conn.query_row(
            "SELECT id FROM properties WHERE name = ?1",
            params![prop_name],
            |r| r.get(0),
        )?;
        conn.execute(
            "INSERT INTO component_properties (component_id, property_id) VALUES (?1, ?2)",
            params![component_id, property_id],
        )?;
    }
    Ok(())
}

pub fn rename_component(conn: &Connection, old_name: &str, new_name: &str) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE components SET name = ?1 WHERE name = ?2",
        params![new_name, old_name],
    )?;
    Ok(())
}

pub fn delete_component(conn: &Connection, name: &str) -> rusqlite::Result<()> {
    let component_id: i64 = conn.query_row(
        "SELECT id FROM components WHERE name = ?1",
        params![name],
        |r| r.get(0),
    )?;
    conn.execute(
        "DELETE FROM component_properties WHERE component_id = ?1",
        params![component_id],
    )?;
    conn.execute("DELETE FROM components WHERE id = ?1", params![component_id])?;
    Ok(())
}

pub fn update_component_properties(
    conn: &Connection,
    component_name: &str,
    prop_names: &[String; 4],
) -> Result<(), String> {
    let component_id: i64 = conn
        .query_row(
            "SELECT id FROM components WHERE name = ?1",
            params![component_name],
            |r| r.get(0),
        )
        .map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM component_properties WHERE component_id = ?1",
        params![component_id],
    )
    .map_err(|e| e.to_string())?;

    for (i, name) in prop_names.iter().enumerate() {
        let name = name.trim();
        if name.is_empty() {
            return Err(format!("свойство {} не может быть пустым", i + 1));
        }
        let property_id: i64 = conn
            .query_row("SELECT id FROM properties WHERE name = ?1", params![name], |r| r.get(0))
            .map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO component_properties (component_id, property_id) VALUES (?1, ?2)",
            params![component_id, property_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

на:

```rust
/// Сохраняет картинку (байты выбранного пользователем файла — либо None,
/// чтобы убрать картинку) и описание компонента. Пишет и в legacy-колонку
/// components.description (источник для бэкфилла migrate_i18n при
/// следующем запуске — см. B1), и напрямую обновляет component_translations
/// для lang='ru' — иначе правка не была бы видна до перезапуска программы.
/// Редактирование сейчас всегда происходит на русском (lang='ru') —
/// поведение при выбранном другом языке не определено этим планом, это
/// отдельная задача (B4, UX редактирования при нескольких языках).
pub fn set_component_media(
    conn: &Connection,
    id: i64,
    image_bytes: Option<&[u8]>,
    description: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE components SET image = ?1, description = ?2 WHERE id = ?3",
        params![image_bytes, description, id],
    )?;
    conn.execute(
        "UPDATE component_translations SET description = ?1 WHERE component_id = ?2 AND lang = 'ru'",
        params![description, id],
    )?;
    Ok(())
}

/// Проверка на дубликат при создании нового компонента — в рамках
/// конкретного языка: создающий ингредиент на английском не должен
/// натыкаться на совпадения с русскими названиями других компонентов.
pub fn component_exists(conn: &Connection, name: &str, lang: &str) -> rusqlite::Result<bool> {
    let id: Option<i64> = conn
        .query_row(
            "SELECT component_id FROM component_translations WHERE lang = ?1 AND name = ?2",
            params![lang, name],
            |r| r.get(0),
        )
        .optional()?;
    Ok(id.is_some())
}

/// Новый компонент, добавленный через "Редактировать базу", всегда
/// помечается Addon::UserAdded явно (не полагаемся на DEFAULT колонки) —
/// это единственный путь, которым в базе вообще может появиться такая
/// пометка (см. addons.rs). Имя пишется и в legacy-колонку components.name
/// (источник, из которого migrate_i18n бэкфиллит lang='ru' при следующем
/// запуске — см. B1), и напрямую в component_translations для lang, на
/// котором ингредиент создан — иначе созданный только что компонент не
/// отображался бы до перезапуска программы. Возвращает id новой записи.
pub fn insert_component(
    conn: &Connection,
    name: &str,
    lang: &str,
    prop_ids: &[i64; 4],
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO components (name, addon) VALUES (?1, ?2)",
        params![name, Addon::UserAdded.as_str()],
    )?;
    let component_id = conn.last_insert_rowid();

    conn.execute(
        "INSERT INTO component_translations (component_id, lang, name, description) VALUES (?1, ?2, ?3, '')",
        params![component_id, lang, name],
    )?;

    for &property_id in prop_ids {
        conn.execute(
            "INSERT INTO component_properties (component_id, property_id) VALUES (?1, ?2)",
            params![component_id, property_id],
        )?;
    }
    Ok(component_id)
}

/// Удаляет компонент вместе со всеми его связями (какие свойства выбраны)
/// и переводами — это делает то же, что миграция migrate_i18n делает при
/// следующем запуске (см. B1, "orphan purge"), но сразу, в той же сессии:
/// без этого повторное создание компонента в этой же сессии (SQLite
/// переиспользует освободившийся rowid) могло бы унаследовать переводы
/// удалённого компонента на других языках раньше следующего перезапуска.
pub fn delete_component(conn: &Connection, id: i64) -> rusqlite::Result<()> {
    conn.execute("DELETE FROM component_properties WHERE component_id = ?1", params![id])?;
    conn.execute("DELETE FROM component_translations WHERE component_id = ?1", params![id])?;
    conn.execute("DELETE FROM components WHERE id = ?1", params![id])?;
    Ok(())
}

/// prop_ids — id уже существующих свойств; какой из 4 слотов пуст/невалиден
/// решает фронтенд (Select не даёт отправить форму без выбора) — эта
/// функция доверяет, что все 4 id валидны.
pub fn update_component_properties(
    conn: &Connection,
    component_id: i64,
    prop_ids: &[i64; 4],
) -> Result<(), String> {
    conn.execute(
        "DELETE FROM component_properties WHERE component_id = ?1",
        params![component_id],
    )
    .map_err(|e| e.to_string())?;

    for &property_id in prop_ids {
        conn.execute(
            "INSERT INTO component_properties (component_id, property_id) VALUES (?1, ?2)",
            params![component_id, property_id],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}
```

`rename_component` не переносится никуда — просто удалите её целиком (была
между `insert_component` и `delete_component` в исходном файле).

- [ ] **Step 4: Убедиться, что компилируется и тесты проходят**

Run: `cd src-tauri && cargo test --lib`
Expected: 17 тестов (15 прежних + 2 новых из Step 1 этой задачи) — `ok`.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat: switch db.rs write functions to id, remove rename_component (Task 2 of B2a)"
```

---

### Task 3: `commands.rs` и регистрация в `lib.rs`

**Files:**
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: все сигнатуры из Task 1 и Task 2 (`db::PropertyInfo`,
  `db::ComponentNameInfo`, `db::PropWithType`, `db::CombinationResult`, все
  функции с `id`/`lang`-параметрами).
- Produces: обновлённая Tauri command surface — потребляется планом B2b
  (фронтенд, следующий план, не в рамках этой задачи).

- [ ] **Step 1: Заменить команды в `commands.rs`**

Замените:

```rust
#[tauri::command]
pub fn get_properties(state: State<AppState>) -> Result<Vec<String>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::load_properties(&conn).map_err(map_err)
}

#[tauri::command]
pub fn get_component_names(state: State<AppState>) -> Result<Vec<String>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::load_component_names(&conn).map_err(map_err)
}

/// Список имён компонентов, ограниченный включёнными в Настройках
/// дополнениями — используется выпадающим списком "Компонент" на основном
/// экране. "Редактировать базу" по-прежнему использует get_component_names
/// (полный список, без фильтра).
#[tauri::command]
pub fn get_component_names_filtered(state: State<AppState>, addons: Vec<Addon>) -> Result<Vec<String>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::load_component_names_filtered(&conn, &addons).map_err(map_err)
}

#[tauri::command]
pub fn get_component_properties(state: State<AppState>, name: String) -> Result<Vec<String>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::component_properties(&conn, &name).map_err(map_err)
}

#[tauri::command]
pub fn get_component_properties_with_types(
    state: State<AppState>,
    name: String,
) -> Result<Vec<db::PropWithType>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::component_properties_with_types(&conn, &name).map_err(map_err)
}
```

на:

```rust
#[tauri::command]
pub fn get_properties(state: State<AppState>, lang: String) -> Result<Vec<db::PropertyInfo>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::load_properties(&conn, &lang).map_err(map_err)
}

#[tauri::command]
pub fn get_component_names(state: State<AppState>, lang: String) -> Result<Vec<db::ComponentNameInfo>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::load_component_names(&conn, &lang).map_err(map_err)
}

/// Список имён компонентов, ограниченный включёнными в Настройках
/// дополнениями — используется выпадающим списком "Компонент" на основном
/// экране. "Редактировать базу" по-прежнему использует get_component_names
/// (полный список, без фильтра).
#[tauri::command]
pub fn get_component_names_filtered(
    state: State<AppState>,
    addons: Vec<Addon>,
    lang: String,
) -> Result<Vec<db::ComponentNameInfo>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::load_component_names_filtered(&conn, &addons, &lang).map_err(map_err)
}

#[tauri::command]
pub fn get_component_properties(state: State<AppState>, id: i64, lang: String) -> Result<Vec<String>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::component_properties(&conn, id, &lang).map_err(map_err)
}

#[tauri::command]
pub fn get_component_properties_with_types(
    state: State<AppState>,
    id: i64,
    lang: String,
) -> Result<Vec<db::PropWithType>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::component_properties_with_types(&conn, id, &lang).map_err(map_err)
}
```

Замените:

```rust
#[tauri::command]
pub fn get_component_media(state: State<AppState>, name: String) -> Result<ComponentMedia, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    let (bytes, description) = db::component_media(&conn, &name);
    Ok(ComponentMedia {
        image_data_url: bytes.map(|b| to_data_url(&b)),
        description,
    })
}

#[tauri::command]
pub fn find_combinations(
    state: State<AppState>,
    selected: Vec<String>,
    filter: String,
    addons: Vec<Addon>,
) -> Result<Vec<db::CombinationResult>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::find_combinations(&conn, &selected, &filter, &addons).map_err(map_err)
}

#[tauri::command]
pub fn find_pairs(
    state: State<AppState>,
    filter: String,
    addons: Vec<Addon>,
    max_results: u32,
) -> Result<Vec<String>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::find_pairs(&conn, &filter, &addons, max_results as usize).map_err(map_err)
}

#[tauri::command]
pub fn find_max_combinations(
    state: State<AppState>,
    filter: String,
    addons: Vec<Addon>,
    max_results: u32,
) -> Result<Vec<String>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::find_max_combinations(&conn, &filter, &addons, max_results as usize).map_err(map_err)
}

#[tauri::command]
pub fn component_exists(state: State<AppState>, name: String) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::component_exists(&conn, &name).map_err(map_err)
}

/// Название/свойства/удаление в "Редактировать базу" разрешены только для
/// компонентов, добавленных самим пользователем (Addon::UserAdded) — у
/// остальных нет безопасного пути восстановления при ошибке, см. заметки
/// про инцидент с "Аронией".
#[tauri::command]
pub fn is_user_added_component(state: State<AppState>, name: String) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    let addon = db::component_addon(&conn, &name).map_err(map_err)?;
    Ok(addon == Some(Addon::UserAdded))
}

#[tauri::command]
pub fn insert_component(
    state: State<AppState>,
    name: String,
    props: [String; 4],
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::insert_component(&conn, &name, &props).map_err(map_err)
}

#[tauri::command]
pub fn rename_component(state: State<AppState>, old_name: String, new_name: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::rename_component(&conn, &old_name, &new_name).map_err(map_err)
}

#[tauri::command]
pub fn delete_component(state: State<AppState>, name: String) -> Result<(), String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::delete_component(&conn, &name).map_err(map_err)
}

#[tauri::command]
pub fn update_component_properties(
    state: State<AppState>,
    name: String,
    props: [String; 4],
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::update_component_properties(&conn, &name, &props)
}

/// image_base64 — «сырой» base64 без префикса data:...;base64, (или None,
/// чтобы убрать картинку компонента), как приходит из pick_image_file/
/// input[type=file] на фронтенде.
#[tauri::command]
pub fn set_component_media(
    state: State<AppState>,
    name: String,
    image_base64: Option<String>,
    description: String,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(map_err)?;
    let bytes = image_base64.map(|s| STANDARD.decode(s)).transpose().map_err(map_err)?;
    db::set_component_media(&conn, &name, bytes.as_deref(), &description).map_err(map_err)
}
```

на:

```rust
#[tauri::command]
pub fn get_component_media(state: State<AppState>, id: i64, lang: String) -> Result<ComponentMedia, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    let (bytes, description) = db::component_media(&conn, id, &lang);
    Ok(ComponentMedia {
        image_data_url: bytes.map(|b| to_data_url(&b)),
        description,
    })
}

#[tauri::command]
pub fn find_combinations(
    state: State<AppState>,
    selected: Vec<i64>,
    filter: String,
    addons: Vec<Addon>,
    lang: String,
) -> Result<Vec<db::CombinationResult>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::find_combinations(&conn, &selected, &filter, &addons, &lang).map_err(map_err)
}

#[tauri::command]
pub fn find_pairs(
    state: State<AppState>,
    filter: String,
    addons: Vec<Addon>,
    max_results: u32,
    lang: String,
) -> Result<Vec<String>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::find_pairs(&conn, &filter, &addons, max_results as usize, &lang).map_err(map_err)
}

#[tauri::command]
pub fn find_max_combinations(
    state: State<AppState>,
    filter: String,
    addons: Vec<Addon>,
    max_results: u32,
    lang: String,
) -> Result<Vec<String>, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::find_max_combinations(&conn, &filter, &addons, max_results as usize, &lang).map_err(map_err)
}

#[tauri::command]
pub fn component_exists(state: State<AppState>, name: String, lang: String) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::component_exists(&conn, &name, &lang).map_err(map_err)
}

/// Название/свойства/удаление в "Редактировать базу" разрешены только для
/// компонентов, добавленных самим пользователем (Addon::UserAdded) — у
/// остальных нет безопасного пути восстановления при ошибке, см. заметки
/// про инцидент с "Аронией".
#[tauri::command]
pub fn is_user_added_component(state: State<AppState>, id: i64) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    let addon = db::component_addon(&conn, id).map_err(map_err)?;
    Ok(addon == Some(Addon::UserAdded))
}

#[tauri::command]
pub fn insert_component(
    state: State<AppState>,
    name: String,
    lang: String,
    props: [i64; 4],
) -> Result<i64, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::insert_component(&conn, &name, &lang, &props).map_err(map_err)
}

#[tauri::command]
pub fn delete_component(state: State<AppState>, id: i64) -> Result<(), String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::delete_component(&conn, id).map_err(map_err)
}

#[tauri::command]
pub fn update_component_properties(
    state: State<AppState>,
    id: i64,
    props: [i64; 4],
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::update_component_properties(&conn, id, &props)
}

/// image_base64 — «сырой» base64 без префикса data:...;base64, (или None,
/// чтобы убрать картинку компонента), как приходит из pick_image_file/
/// input[type=file] на фронтенде.
#[tauri::command]
pub fn set_component_media(
    state: State<AppState>,
    id: i64,
    image_base64: Option<String>,
    description: String,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(map_err)?;
    let bytes = image_base64.map(|s| STANDARD.decode(s)).transpose().map_err(map_err)?;
    db::set_component_media(&conn, id, bytes.as_deref(), &description).map_err(map_err)
}
```

- [ ] **Step 2: Убрать `rename_component` из регистрации команд**

В `src-tauri/src/lib.rs`, внутри `tauri::generate_handler![...]`, удалить
строку:

```rust
            commands::rename_component,
```

(находится между `commands::insert_component,` и `commands::delete_component,`).

- [ ] **Step 3: Убедиться, что Rust-крейт компилируется целиком**

Run: `cd src-tauri && cargo check`
Expected: без ошибок (frontend НЕ проверяется этой командой — `npm run
build` в этот момент ожидаемо не пройдёт, это нормальное промежуточное
состояние до плана B2b, см. Global Constraints).

- [ ] **Step 4: Прогнать полный набор тестов**

Run: `cd src-tauri && cargo test --lib`
Expected: 17 тестов — `ok`, без предупреждений.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: update commands.rs to id + lang, remove rename_component command (Task 3 of B2a)"
```

---

## Итоговая проверка плана

- [ ] `cargo check` и `cargo test --lib` (из `src-tauri/`) — оба зелёные,
  без предупреждений (кроме уже существующих до этого плана
  `TYPE_BENEFIT`/`TYPE_POISON` — не трогать, не в рамках этого плана).
- [ ] `rename_component` отсутствует в `db.rs`, `commands.rs` и в
  `generate_handler!` в `lib.rs`.
- [ ] Ни один файл в `src/` (фронтенд) не тронут этим планом — это
  осознанно отложено до B2b.
- [ ] `npm run build`/`tsc -b` в этот момент падают — ожидаемо, не признак
  ошибки в этом плане.
