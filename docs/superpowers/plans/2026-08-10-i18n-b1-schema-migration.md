# Схема БД и миграция переводов (План B1) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** добавить в БД таблицы `component_translations`/`property_translations`
и миграцию `migrate_i18n()`, которая наполняет их официальными переводами
(из уже готового `tools/i18n`-пайплайна) на 8 языков плюс бэкфиллит `ru` из
текущего содержимого БД — без изменения поведения ни одной существующей
функции.

**Architecture:** три небольшие, последовательные задачи: (1) поправить код
языка `zh` → `zh-Hant` в пайплайне и перегенерировать
`src-tauri/src/seed_translations.rs`; (2) сама миграция в `db.rs`,
подключённая в конец цепочки `ensure_db()`; (3) синхронизировать код языка
во фронтенде (`SettingsModal.tsx`). Ни один существующий `db.rs`/`commands.rs`
код не меняется — задача читает новые таблицы только в тестах.

**Tech Stack:** Rust/rusqlite (существующий стек проекта), Python (уже
существующий `tools/i18n`-пайплайн из плана A, точечная правка).

## Global Constraints

- Схема (уже утверждена в `docs/superpowers/specs/2026-08-10-i18n-storage-design.md`):
  ```sql
  CREATE TABLE component_translations (
      component_id INTEGER NOT NULL,
      lang         TEXT NOT NULL,
      name         TEXT NOT NULL,
      description  TEXT NOT NULL DEFAULT '',
      PRIMARY KEY (component_id, lang)
  );
  CREATE TABLE property_translations (
      property_id INTEGER NOT NULL,
      lang        TEXT NOT NULL,
      name        TEXT NOT NULL,
      PRIMARY KEY (property_id, lang)
  );
  ```
- Код китайского языка — `zh-Hant` (Традиционный китайский), не `zh` —
  решение по итогам финального ревью плана A. Меняется и в пайплайне
  (`tools/i18n/match_translations.py`), и во фронтенде
  (`SettingsModal.tsx`).
- `TRANSLATIONS` (`src-tauri/src/seed_translations.rs`) **не содержит**
  записей с `lang == "ru"` — по построению: `tools/i18n/match_translations.py`
  генерируется с явным списком языков на входе, и `"ru"` в этот список
  никогда не передаётся (сам русский текст — это то, с чем сопоставляют, а
  не то, что переводят). `ru`-строки в таблицах переводов всегда берутся
  из **текущего** содержимого `components`/`properties`, не из
  `TRANSLATIONS` — это единственный способ не потерять уже внесённые
  пользователем правки описаний.
- `TRANSLATIONS` не различает "это компонент" и "это свойство" — плоский
  список троек `(ru_name, lang, text)`. Различать нужно при вставке, по
  наличию `ru_name` в `components.name` либо в `properties.name` — эти два
  множества имён не пересекаются (проверено на текущих данных).
- Никакая существующая функция в `db.rs`/`commands.rs` не должна менять
  поведение или начать читать новые таблицы — это задача плана B2. Эта
  задача только создаёт и наполняет данные.
- Синтаксис БД и стиль тестов — точно как у существующих `migrate_*` в
  `db.rs` (смотри `migrate_addon_ids`, `migrate_rare_curios`, и тесты в
  `#[cfg(test)] mod rare_curios_tests`) — новые тесты добавляются в этот же
  модуль, не создают новый.

---

### Task 1: Код языка `zh-Hant` и перегенерация `seed_translations.rs`

**Files:**
- Modify: `tools/i18n/match_translations.py`
- Create: `src-tauri/src/seed_translations.rs` (сгенерированные данные,
  коммитится как обычный файл — не пересобирается при `cargo build`, точно
  как `seed_data.rs`)

**Interfaces:**
- Produces: `pub const TRANSLATIONS: &[(&str, &str, &str)]` — тройки
  `(ru_name, lang, translated_text)` для всех языков кроме `ru` (см. Global
  Constraints). Потребляется в Task 2.

- [ ] **Step 1: Поменять код языка в `LANG_SUFFIX`**

В `tools/i18n/match_translations.py` заменить:
```python
LANG_SUFFIX = {
    "ru": "russian",
    "en": "english",
    "fr": "french",
    "de": "german",
    "it": "italian",
    "es": "spanish",
    "pl": "polish",
    "ja": "japanese",
    "zh": "chinese",
}
```
на:
```python
LANG_SUFFIX = {
    "ru": "russian",
    "en": "english",
    "fr": "french",
    "de": "german",
    "it": "italian",
    "es": "spanish",
    "pl": "polish",
    "ja": "japanese",
    "zh-Hant": "chinese",  # Bethesda's Chinese localization is Traditional
    # Chinese, not generic "Chinese" — see 2026-08-10 final review of Plan A.
}
```

- [ ] **Step 2: Переименовать папку с уже скачанными файлами**

Run: `mv "$HOME/Рабочий стол/alchemist-tauri-strings-src/zh" "$HOME/Рабочий стол/alchemist-tauri-strings-src/zh-Hant"`

- [ ] **Step 3: Перегенерировать пайплайн**

Run (из `tools/i18n/`):
```bash
python3 extract_ru_names.py ../.. > ru_names.json
python3 match_translations.py ru_names.json "$HOME/Рабочий стол/alchemist-tauri-strings-src" en,fr,de,it,es,pl,ja,zh-Hant > translations.json
python3 generate_seed_translations.py translations.json > seed_translations.rs
```
Expected (в stderr от `match_translations.py`): `Сопоставлено: 238/239` —
то же число, что и в плане A (переименование языка не меняет логику
сопоставления, только его код).

- [ ] **Step 4: Проверить, что `zh-Hant` реально попал в вывод**

Run:
```bash
grep -c '"zh-Hant"' seed_translations.rs
grep -c '"zh"' seed_translations.rs
```
Expected: первая команда — `238`, вторая — `0` (старый код языка нигде не
остался).

- [ ] **Step 5: Скопировать в крейт и закоммитить**

Run:
```bash
cp tools/i18n/seed_translations.rs src-tauri/src/seed_translations.rs
git add tools/i18n/match_translations.py src-tauri/src/seed_translations.rs
git commit -m "feat: switch zh to zh-Hant, commit official name translations"
```

---

### Task 2: `migrate_i18n()` — создание таблиц и наполнение

**Files:**
- Modify: `src-tauri/src/lib.rs` (регистрация нового модуля)
- Modify: `src-tauri/src/db.rs` (сама миграция + вызов из `ensure_db` + тесты)

**Interfaces:**
- Consumes: `crate::seed_translations::TRANSLATIONS` из Task 1.
- Produces: таблицы `component_translations`/`property_translations`,
  заполненные при каждом вызове `ensure_db()`. Публичных функций для
  фронтенда/команд эта задача не добавляет — это план B2.

- [ ] **Step 1: Зарегистрировать новый модуль**

В `src-tauri/src/lib.rs`, рядом с остальными `mod`:
```rust
mod seed_translations;
```
(строка добавляется в алфавитном порядке существующего списка модулей —
после `mod seed_data;`, если он идёт последним, иначе на своё
алфавитное место рядом с остальными).

- [ ] **Step 2: Написать проваливающиеся тесты**

В `src-tauri/src/db.rs`, внутри `#[cfg(test)] mod rare_curios_tests`
(существующий тестовый модуль — новые тесты добавляются рядом с уже
существующими, не в новый модуль), добавить:

```rust
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
    let (_, expected_description) = component_media(&conn, "Белянка");
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

#[test]
fn migration_populates_translations_from_seed_translations() {
    let db_path = temp_db_path("i18n_translations");
    let _ = std::fs::remove_file(&db_path);
    let conn = ensure_db(&db_path).unwrap();

    let property_id: i64 = conn
        .query_row("SELECT id FROM properties WHERE name = ?1", params!["Бешенство"], |r| r.get(0))
        .unwrap();
    let en_name: String = conn
        .query_row(
            "SELECT name FROM property_translations WHERE property_id = ?1 AND lang = 'en'",
            params![property_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(en_name, "Frenzy");

    let component_id: i64 = conn
        .query_row("SELECT id FROM components WHERE name = ?1", params!["Стеклянный окунь"], |r| r.get(0))
        .unwrap();
    let en_component_name: String = conn
        .query_row(
            "SELECT name FROM component_translations WHERE component_id = ?1 AND lang = 'en'",
            params![component_id],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(en_component_name, "Glassfish");

    drop(conn);
    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn migration_skips_name_with_no_official_translation() {
    let db_path = temp_db_path("i18n_unmatched");
    let _ = std::fs::remove_file(&db_path);
    let conn = ensure_db(&db_path).unwrap();

    let component_id: i64 = conn
        .query_row("SELECT id FROM components WHERE name = ?1", params!["Смертная плоть"], |r| r.get(0))
        .unwrap();
    let translation_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM component_translations WHERE component_id = ?1",
            params![component_id],
            |r| r.get(0),
        )
        .unwrap();
    // Только ru-бэкфилл — плагин Plague of the Dead отсутствует в
    // скачанном архиве официальных строк, перевода на другие языки нет.
    assert_eq!(translation_count, 1);

    drop(conn);
    let _ = std::fs::remove_file(&db_path);
}

#[test]
fn migration_is_idempotent() {
    let db_path = temp_db_path("i18n_idempotent");
    let _ = std::fs::remove_file(&db_path);
    let conn = ensure_db(&db_path).unwrap();

    let count_before: i64 = conn
        .query_row("SELECT COUNT(*) FROM component_translations", [], |r| r.get(0))
        .unwrap();
    let prop_count_before: i64 = conn
        .query_row("SELECT COUNT(*) FROM property_translations", [], |r| r.get(0))
        .unwrap();
    assert!(count_before > 200, "подозрительно мало строк переводов компонентов после миграции");

    drop(conn);
    let conn2 = ensure_db(&db_path).unwrap();

    let count_after: i64 = conn2
        .query_row("SELECT COUNT(*) FROM component_translations", [], |r| r.get(0))
        .unwrap();
    let prop_count_after: i64 = conn2
        .query_row("SELECT COUNT(*) FROM property_translations", [], |r| r.get(0))
        .unwrap();

    assert_eq!(count_before, count_after, "повторная миграция задвоила component_translations");
    assert_eq!(prop_count_before, prop_count_after, "повторная миграция задвоила property_translations");

    drop(conn2);
    let _ = std::fs::remove_file(&db_path);
}
```

- [ ] **Step 3: Убедиться, что тесты падают**

Run: `cd src-tauri && cargo test --lib i18n`
Expected: код скомпилируется (SQL — это просто строка, `rusqlite` проверяет
её только в рантайме), но все 4 новых теста упадут в рантайме с паникой на
`.unwrap()` — `SqliteFailure(..., Some("no such table: component_translations"))`
(таблиц ещё нет, потому что `migrate_i18n` ещё не написана и не подключена
к `ensure_db`).

- [ ] **Step 4: Реализовать `migrate_i18n()`**

В `src-tauri/src/db.rs`, добавить импорт рядом с остальными `use crate::...`:
```rust
use crate::seed_translations::TRANSLATIONS;
```

Добавить функцию (рядом с остальными `migrate_*`, например сразу после
`migrate_images_to_blob`):
```rust
/// Создаёт таблицы переводов (если их ещё нет) и наполняет их: сначала
/// бэкфиллит lang='ru' из ТЕКУЩЕГО содержимого components/properties (то
/// есть уже с учётом правок, внесённых пользователем через "Редактировать
/// базу"), затем — из TRANSLATIONS (см. seed_translations.rs, сгенерирован
/// tools/i18n) для остальных языков. TRANSLATIONS не содержит записей с
/// lang == "ru" по построению (см. Global Constraints плана B1) — только
/// не-русские языки.
///
/// TRANSLATIONS — плоский список троек (ru_name, lang, text) без разметки
/// "компонент/свойство"; принадлежность определяется тем, в какой из двух
/// таблиц (components/properties) нашлось совпадение по имени — эти два
/// множества имён не пересекаются.
///
/// Имя без соответствия в текущей БД (например "Смертная плоть" — плагин
/// Plague of the Dead отсутствует в скачанном архиве официальных строк)
/// пропускается без падения, как и в остальных migrate_*.
fn migrate_i18n(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS component_translations (
            component_id INTEGER NOT NULL,
            lang         TEXT NOT NULL,
            name         TEXT NOT NULL,
            description  TEXT NOT NULL DEFAULT '',
            PRIMARY KEY (component_id, lang)
        );
        CREATE TABLE IF NOT EXISTS property_translations (
            property_id INTEGER NOT NULL,
            lang        TEXT NOT NULL,
            name        TEXT NOT NULL,
            PRIMARY KEY (property_id, lang)
        );",
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO component_translations (component_id, lang, name, description)
         SELECT id, 'ru', name, description FROM components",
        [],
    )?;
    conn.execute(
        "INSERT OR IGNORE INTO property_translations (property_id, lang, name)
         SELECT id, 'ru', name FROM properties",
        [],
    )?;

    for (ru_name, lang, text) in TRANSLATIONS {
        let component_id: Option<i64> = conn
            .query_row("SELECT id FROM components WHERE name = ?1", params![ru_name], |r| r.get(0))
            .optional()?;
        if let Some(id) = component_id {
            conn.execute(
                "INSERT OR IGNORE INTO component_translations (component_id, lang, name, description)
                 VALUES (?1, ?2, ?3, '')",
                params![id, lang, text],
            )?;
            continue;
        }

        let property_id: Option<i64> = conn
            .query_row("SELECT id FROM properties WHERE name = ?1", params![ru_name], |r| r.get(0))
            .optional()?;
        if let Some(id) = property_id {
            conn.execute(
                "INSERT OR IGNORE INTO property_translations (property_id, lang, name)
                 VALUES (?1, ?2, ?3)",
                params![id, lang, text],
            )?;
        }
    }

    Ok(())
}
```

Подключить в конец цепочки миграций в `ensure_db`:
```rust
    migrate_media(&conn)?;
    migrate_rare_curios(&conn)?;
    migrate_creations(&conn)?;
    migrate_addon_ids(&conn)?;
    migrate_images_to_blob(&conn)?;
    migrate_i18n(&conn)?;
    Ok(conn)
```
(именно последней — ей нужны уже вставленные по имени компоненты/свойства
от всех предыдущих миграций).

- [ ] **Step 5: Убедиться, что тесты проходят**

Run: `cd src-tauri && cargo test --lib i18n`
Expected: все 4 новых теста — `ok`.

- [ ] **Step 6: Прогнать полный набор тестов**

Run: `cd src-tauri && cargo test --lib`
Expected: все тесты (старые + новые 4) зелёные, без падений и без новых
warning'ов.

---

### Task 3: Код языка `zh-Hant` во фронтенде

**Files:**
- Modify: `src/components/SettingsModal.tsx`

**Interfaces:**
- Ничего не потребляет и не производит для других задач — чисто
  синхронизация значения с Task 1, до того как язык реально станет
  переключаемым (это план B3).

- [ ] **Step 1: Поменять значение в `LANGUAGE_OPTIONS`**

В `src/components/SettingsModal.tsx`, строка с `zh`:
```tsx
  { value: "zh", label: "🇨🇳 中文" },
```
заменить на:
```tsx
  { value: "zh-Hant", label: "🇨🇳 中文" },
```

- [ ] **Step 2: Проверить сборку**

Run: `npm run build`
Expected: сборка проходит без ошибок (значение `zh-Hant` пока нигде не
читается логикой — оно используется только как значение статичного
`<Select>`, реального переключения языка ещё нет).

---

## Итоговая проверка плана

- [ ] `cargo test --lib` — все тесты зелёные (существующие + 4 новых).
- [ ] `cargo check` и `npm run build` — без ошибок.
- [ ] `grep -c '"zh-Hant"' src-tauri/src/seed_translations.rs` — 238,
  `grep -c '"zh"' src-tauri/src/seed_translations.rs` — 0.
- [ ] Ни один файл вне `tools/i18n/`, `src-tauri/src/{lib.rs,db.rs,seed_translations.rs}`
  и `src/components/SettingsModal.tsx` не тронут — план B1 не меняет
  поведение существующего кода.
