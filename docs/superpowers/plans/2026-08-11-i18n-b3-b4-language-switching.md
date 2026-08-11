# Реальное переключение языка (Планы B3+B4) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** сделать переключение языка в Настройках реально работающим —
`language` становится настоящим persisted state (не захардкоженной
константой `CURRENT_LANG`), с предупреждением при смене языка, если у
пользователя есть свои ингредиенты, и минимальной подсказкой в редакторе
базы о том, что описание/новый ингредиент видны только на текущем языке.

**Architecture:** шесть последовательных задач. Task 1-2 — бэкенд
(`db.rs`/`layout.rs`/`commands.rs`/`lib.rs`), каждая оставляет
`cargo check`/`cargo test --lib` зелёными. Task 3-6 — фронтенд, в порядке
зависимостей: `api.ts` (общий контракт) → `lib/languages.ts` +
`SettingsModal.tsx` → `EditorModal.tsx` → `App.tsx` (финальная склейка,
подключает пропы обоих предыдущих компонентов). Промежуточные состояния
внутри плана не обязаны собираться на 100% (как и в B2a/B2b) — полностью
зелёный `npm run build` ожидается только после Task 6.

**Tech Stack:** Rust/rusqlite (бэкенд, есть тесты), TypeScript/React/Mantine
(фронтенд, тестового раннера в проекте нет — верификация через `npx tsc -b`
+ `npm run build` + `npm run lint`, как в B2b).

## Global Constraints

- **Видимость пользовательских ингредиентов:** вариант (b) — ингредиент
  виден только на языке, на котором создан, **везде**, без исключений (в
  том числе в списке "Ингредиент" внутри "Редактировать базу" — уже так
  устроено текущим `INNER JOIN` в `db.rs`, этот план ничего не меняет в
  логике чтения). `LEFT JOIN`+`COALESCE`-вариант (a) — явно вне рамок,
  ничего в этом плане не готовит почву под него отдельно.
- **`migrate_i18n()` перестаёт трогать `Addon::UserAdded`-компоненты** — ни
  бэкфилл `lang='ru'` из legacy-колонок, ни стирание не-`ru` строк. Их
  переводы полностью на совести `insert_component`/`set_component_media`
  (уже пишут напрямую в `component_translations` под нужным `lang`, без
  изменений в этом плане).
- **Текст интерфейса остаётся на русском** — предупреждение при смене
  языка и строка в диалоге "Новый ингредиент" **не переводятся** на 9
  языков. Полный перевод UI — отдельная будущая задача, в этом плане не
  начинается ни архитектурно, ни контентно.
- **Механизм самовосстановления БД при каждом запуске (`ensure_db()`)** —
  не трогается, остаётся как есть (обсуждали отдельно, решили не убирать).
- Схема `Layout`/`layout.rs` уже имеет прецедент для persisted-полей со
  своей save-функцией (`scale`/`save_scale`, `enabled_addons`/`save_addons`,
  `max_combinations`/`save_max_combinations`) — новое поле `language`
  повторяет этот же паттерн 1-в-1.
- **Известная особенность окружения (не баг этого плана):** `lib.rs`'s
  `tauri::generate_context!()` требует, чтобы `../dist` физически
  существовал — если в рабочей копии давно не было `npm run build`,
  выполните его один раз в корне проекта, иначе первый же `cargo check`
  в Task 1 упадёт с ошибкой про `frontendDist`, не связанной с этим планом.

---

### Task 1: `db.rs` — `has_user_added_components()` + фикс `migrate_i18n()`

**Files:**
- Modify: `src-tauri/src/db.rs`

**Interfaces:**
- Consumes: `Addon::UserAdded` (`crate::addons::Addon`, уже импортирован в
  `db.rs`).
- Produces (используется в Task 2, `commands.rs`):
  - `pub fn has_user_added_components(conn: &Connection) -> rusqlite::Result<bool>`

- [ ] **Step 1: Добавить тесты (RED — не скомпилируется, `has_user_added_components` ещё не существует)**

В `#[cfg(test)] mod rare_curios_tests`, сразу после теста
`insert_component_is_tagged_user_added` (заканчивается `drop(conn); let _ =
std::fs::remove_file(&db_path); }` перед комментарием "Final review finding
#1"), добавьте два новых теста:

```rust
    #[test]
    fn has_user_added_components_reflects_state() {
        let db_path = temp_db_path("has_user_added");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        assert!(
            !has_user_added_components(&conn).unwrap(),
            "свежая база не должна иметь пользовательских ингредиентов"
        );

        let props = [
            property_id_by_name(&conn, "Бешенство"),
            property_id_by_name(&conn, "Замедление"),
            property_id_by_name(&conn, "Страх"),
            property_id_by_name(&conn, "Паралич"),
        ];
        let new_id = insert_component(&conn, "Тестовый ингредиент", "ru", &props).unwrap();
        assert!(has_user_added_components(&conn).unwrap());

        delete_component(&conn, new_id).unwrap();
        assert!(!has_user_added_components(&conn).unwrap());

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    /// Два момента сразу: (1) migrate_i18n не должна путать легаси-колонку
    /// components.name пользовательского компонента, созданного не на
    /// русском, с "русским переводом" — иначе компонент стал бы (неверно)
    /// видимым в ru-списке под нерусским названием; (2) не-ru строка
    /// перевода, созданная insert_component напрямую, не должна стираться
    /// стирающим шагом миграции для lang <> 'ru' (раньше стирала).
    #[test]
    fn migrate_i18n_does_not_touch_user_added_translations() {
        let db_path = temp_db_path("i18n_user_added_untouched");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let props = [
            property_id_by_name(&conn, "Бешенство"),
            property_id_by_name(&conn, "Замедление"),
            property_id_by_name(&conn, "Страх"),
            property_id_by_name(&conn, "Паралич"),
        ];
        let new_id = insert_component(&conn, "Test Ingredient", "en", &props).unwrap();

        let ru_row: Option<String> = conn
            .query_row(
                "SELECT name FROM component_translations WHERE component_id = ?1 AND lang = 'ru'",
                params![new_id],
                |r| r.get(0),
            )
            .optional()
            .unwrap();
        assert_eq!(
            ru_row, None,
            "migrate_i18n не должна создавать ru-перевод из legacy-колонки для пользовательского компонента"
        );

        drop(conn);
        let conn2 = ensure_db(&db_path).unwrap();

        let en_name: String = conn2
            .query_row(
                "SELECT name FROM component_translations WHERE component_id = ?1 AND lang = 'en'",
                params![new_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(en_name, "Test Ingredient", "en-перевод пользовательского компонента не должен стираться");

        drop(conn2);
        let _ = std::fs::remove_file(&db_path);
    }
```

- [ ] **Step 2: Убедиться, что не компилируется**

Run: `cd src-tauri && cargo test --lib has_user_added_components_reflects_state`
Expected: ошибка компиляции — `has_user_added_components` не найдена в этой
области видимости.

- [ ] **Step 3: Реализовать `has_user_added_components`**

Добавьте новую функцию сразу после `component_addon` (после закрывающей `}`
на строке 609, перед комментарием `/// Сохраняет картинку...` у
`set_component_media`):

```rust
/// Есть ли хотя бы один пользовательский ингредиент (Addon::UserAdded) —
/// используется перед предупреждением о смене языка (см. план B3/B4): такие
/// ингредиенты видны только на языке, на котором были созданы, и при смене
/// языка пропадают из списков.
pub fn has_user_added_components(conn: &Connection) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS(SELECT 1 FROM components WHERE addon = ?1)",
        params![Addon::UserAdded.as_str()],
        |r| r.get(0),
    )
}
```

- [ ] **Step 4: Исправить `migrate_i18n` — исключить `Addon::UserAdded`**

Замените (внутри `migrate_i18n`, между `DELETE FROM ... NOT IN (SELECT id
FROM properties)` и циклом `for (ru_name, lang, text) in TRANSLATIONS`):

```rust
    tx.execute(
        "INSERT INTO component_translations (component_id, lang, name, description)
         SELECT id, 'ru', name, description FROM components WHERE true
         ON CONFLICT(component_id, lang) DO UPDATE SET name = excluded.name, description = excluded.description",
        [],
    )?;
    tx.execute(
        "INSERT INTO property_translations (property_id, lang, name)
         SELECT id, 'ru', name FROM properties WHERE true
         ON CONFLICT(property_id, lang) DO UPDATE SET name = excluded.name",
        [],
    )?;

    tx.execute("DELETE FROM component_translations WHERE lang <> 'ru'", [])?;
    tx.execute("DELETE FROM property_translations WHERE lang <> 'ru'", [])?;

    let mut component_ids: HashMap<String, i64> = HashMap::new();
    {
        let mut stmt = tx.prepare("SELECT id, name FROM components")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            component_ids.insert(row.get(1)?, row.get(0)?);
        }
    }
```

на:

```rust
    // Addon::UserAdded исключён из обоих шагов ниже (бэкфилл ru и стирание
    // не-ru) — см. план B3/B4: legacy-колонки components.name/description
    // для такого компонента могут содержать текст НЕ на русском (если
    // insert_component/set_component_media вызывались под другим активным
    // языком), и слепой бэкфилл ошибочно записал бы этот текст как "русский
    // перевод". Переводы Addon::UserAdded полностью на совести
    // insert_component/set_component_media — они уже пишут напрямую в
    // component_translations под нужным lang.
    tx.execute(
        "INSERT INTO component_translations (component_id, lang, name, description)
         SELECT id, 'ru', name, description FROM components WHERE addon != ?1
         ON CONFLICT(component_id, lang) DO UPDATE SET name = excluded.name, description = excluded.description",
        params![Addon::UserAdded.as_str()],
    )?;
    tx.execute(
        "INSERT INTO property_translations (property_id, lang, name)
         SELECT id, 'ru', name FROM properties WHERE true
         ON CONFLICT(property_id, lang) DO UPDATE SET name = excluded.name",
        [],
    )?;

    tx.execute(
        "DELETE FROM component_translations
         WHERE lang <> 'ru' AND component_id NOT IN (SELECT id FROM components WHERE addon = ?1)",
        params![Addon::UserAdded.as_str()],
    )?;
    tx.execute("DELETE FROM property_translations WHERE lang <> 'ru'", [])?;

    let mut component_ids: HashMap<String, i64> = HashMap::new();
    {
        let mut stmt = tx.prepare("SELECT id, name FROM components WHERE addon != ?1")?;
        let mut rows = stmt.query(params![Addon::UserAdded.as_str()])?;
        while let Some(row) = rows.next()? {
            component_ids.insert(row.get(1)?, row.get(0)?);
        }
    }
```

- [ ] **Step 5: Обновить два устаревших doc-комментария**

Замените doc-комментарий у `set_component_media` (начинается `/// Сохраняет
картинку...`, заканчивается `/// Constraints плана B2a...`):

```rust
/// Сохраняет картинку (байты выбранного пользователем файла — либо None,
/// чтобы убрать картинку) и описание компонента. Пишет и в legacy-колонку
/// components.description (источник для бэкфилла migrate_i18n при
/// следующем запуске — см. B1), и напрямую делает upsert в
/// component_translations для переданного lang — иначе правка не была бы
/// видна до перезапуска программы. lang — настоящий параметр (см. Global
/// Constraints плана B2a, никакого хардкода внутри Rust-кода); используется
```

на:

```rust
/// Сохраняет картинку (байты выбранного пользователем файла — либо None,
/// чтобы убрать картинку) и описание компонента. Пишет и в legacy-колонку
/// components.description (источник бэкфилла migrate_i18n для lang='ru' —
/// см. B1 — но только для официальных компонентов: у Addon::UserAdded
/// migrate_i18n эту колонку с плана B3 игнорирует, см. её комментарий), и
/// напрямую делает upsert в component_translations для переданного lang —
/// иначе правка не была бы видна до перезапуска программы. lang — настоящий
/// параметр (см. Global Constraints плана B2a, никакого хардкода внутри
/// Rust-кода); используется
```

Замените doc-комментарий у `insert_component` (начинается `/// Новый
компонент, добавленный через...`, заканчивается `/// Возвращает id новой
записи.`):

```rust
/// Новый компонент, добавленный через "Редактировать базу", всегда
/// помечается Addon::UserAdded явно (не полагаемся на DEFAULT колонки) —
/// это единственный путь, которым в базе вообще может появиться такая
/// пометка (см. addons.rs). Имя пишется и в legacy-колонку components.name
/// (источник, из которого migrate_i18n бэкфиллит lang='ru' при следующем
/// запуске — см. B1), и напрямую в component_translations для lang, на
/// котором ингредиент создан — иначе созданный только что компонент не
/// отображался бы до перезапуска программы. Возвращает id новой записи.
```

на:

```rust
/// Новый компонент, добавленный через "Редактировать базу", всегда
/// помечается Addon::UserAdded явно (не полагаемся на DEFAULT колонки) —
/// это единственный путь, которым в базе вообще может появиться такая
/// пометка (см. addons.rs). Имя пишется и в legacy-колонку components.name
/// (для официальных компонентов — источник, из которого migrate_i18n
/// бэкфиллит lang='ru', см. B1; для Addon::UserAdded с плана B3
/// migrate_i18n эту колонку игнорирует — легаси-запись остаётся чисто
/// историческим артефактом), и напрямую в component_translations для lang,
/// на котором ингредиент создан — иначе созданный только что компонент не
/// отображался бы до перезапуска программы. Возвращает id новой записи.
```

- [ ] **Step 6: Убедиться, что компилируется и тесты проходят**

Run: `cd src-tauri && cargo test --lib`
Expected: все тесты (23 = 21 текущих + 2 новых) — `ok`.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/db.rs
git commit -m "feat: add has_user_added_components, exclude UserAdded from migrate_i18n (Task 1 of B3+B4)"
```

---

### Task 2: `layout.rs` + `commands.rs` + `lib.rs` — `language` поле и новые команды

**Files:**
- Modify: `src-tauri/src/layout.rs`
- Modify: `src-tauri/src/commands.rs`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: `db::has_user_added_components` (Task 1).
- Produces (используется в Task 3, `api.ts`):
  - `Layout.language: String` (сериализуется как `language` в JSON)
  - `pub fn layout::save_language(language: &str)`
  - `#[tauri::command] fn save_language(language: String)`
  - `#[tauri::command] fn has_user_added_components(state) -> Result<bool, String>`

- [ ] **Step 1: `layout.rs` — добавить поле `language`**

Замените:

```rust
const SCALE_OPTIONS: &[&str] = &["Мелкий", "Нормальный", "Крупный"];

#[derive(Serialize, Deserialize, Clone)]
pub struct Layout {
    pub side_panel_width: f64,
    pub split_ratio: f64,
    #[serde(default = "default_scale")]
    pub scale: String,
    /// Сырые строковые идентификаторы (Addon::as_str), а не Vec<Addon> —
    /// чтобы одно неизвестное/устаревшее значение в settings.json (ручная
    /// правка, более старая версия программы) не роняло разбор всего файла
    /// через serde, а просто отфильтровывалось в load() ниже.
    #[serde(default = "default_addons")]
    pub enabled_addons: Vec<String>,
    /// Ограничение числа результатов у "Парных сочетаний" и "Тройных
    /// сочетаний" (см. SettingsModal.tsx, поле "Макс. кол-во сочетаний").
    #[serde(default = "default_max_combinations")]
    pub max_combinations: u32,
}

fn default_scale() -> String {
    "Нормальный".to_string()
}
```

на:

```rust
const SCALE_OPTIONS: &[&str] = &["Мелкий", "Нормальный", "Крупный"];
/// Коды языков — те же, что в SettingsModal.tsx's LANGUAGE_OPTIONS /
/// component_translations.lang (zh-Hant, не zh — см. i18n-storage-design.md).
const LANGUAGE_OPTIONS: &[&str] = &["ru", "en", "fr", "de", "it", "es", "pl", "ja", "zh-Hant"];

#[derive(Serialize, Deserialize, Clone)]
pub struct Layout {
    pub side_panel_width: f64,
    pub split_ratio: f64,
    #[serde(default = "default_scale")]
    pub scale: String,
    /// Сырые строковые идентификаторы (Addon::as_str), а не Vec<Addon> —
    /// чтобы одно неизвестное/устаревшее значение в settings.json (ручная
    /// правка, более старая версия программы) не роняло разбор всего файла
    /// через serde, а просто отфильтровывалось в load() ниже.
    #[serde(default = "default_addons")]
    pub enabled_addons: Vec<String>,
    /// Ограничение числа результатов у "Парных сочетаний" и "Тройных
    /// сочетаний" (см. SettingsModal.tsx, поле "Макс. кол-во сочетаний").
    #[serde(default = "default_max_combinations")]
    pub max_combinations: u32,
    /// Текущий выбранный язык (план B3) — раньше был захардкожен константой
    /// CURRENT_LANG на фронтенде, теперь настоящий persisted state.
    #[serde(default = "default_language")]
    pub language: String,
}

fn default_scale() -> String {
    "Нормальный".to_string()
}

fn default_language() -> String {
    "ru".to_string()
}
```

- [ ] **Step 2: `layout.rs` — обновить `Default`/`load()`/добавить `save_language`**

Замените:

```rust
impl Default for Layout {
    fn default() -> Self {
        Self {
            side_panel_width: 300.0,
            split_ratio: 0.25,
            scale: default_scale(),
            enabled_addons: default_addons(),
            max_combinations: default_max_combinations(),
        }
    }
}

pub fn load() -> Layout {
    let path = crate::paths::settings_path();
    let Ok(data) = std::fs::read_to_string(path) else {
        return Layout::default();
    };
    let Ok(l) = serde_json::from_str::<Layout>(&data) else {
        return Layout::default();
    };
    let default = Layout::default();
    let known_addons: Vec<String> = l
        .enabled_addons
        .iter()
        .filter(|s| Addon::from_id(s).is_some())
        .cloned()
        .collect();
    let enabled_addons = if known_addons.is_empty() { default.enabled_addons.clone() } else { known_addons };
    Layout {
        side_panel_width: if l.side_panel_width >= 150.0 { l.side_panel_width } else { default.side_panel_width },
        split_ratio: if (0.05..=0.95).contains(&l.split_ratio) { l.split_ratio } else { default.split_ratio },
        scale: if SCALE_OPTIONS.contains(&l.scale.as_str()) { l.scale } else { default.scale },
        enabled_addons,
        max_combinations: if l.max_combinations >= 1 { l.max_combinations } else { default.max_combinations },
    }
}
```

на:

```rust
impl Default for Layout {
    fn default() -> Self {
        Self {
            side_panel_width: 300.0,
            split_ratio: 0.25,
            scale: default_scale(),
            enabled_addons: default_addons(),
            max_combinations: default_max_combinations(),
            language: default_language(),
        }
    }
}

pub fn load() -> Layout {
    let path = crate::paths::settings_path();
    let Ok(data) = std::fs::read_to_string(path) else {
        return Layout::default();
    };
    let Ok(l) = serde_json::from_str::<Layout>(&data) else {
        return Layout::default();
    };
    let default = Layout::default();
    let known_addons: Vec<String> = l
        .enabled_addons
        .iter()
        .filter(|s| Addon::from_id(s).is_some())
        .cloned()
        .collect();
    let enabled_addons = if known_addons.is_empty() { default.enabled_addons.clone() } else { known_addons };
    Layout {
        side_panel_width: if l.side_panel_width >= 150.0 { l.side_panel_width } else { default.side_panel_width },
        split_ratio: if (0.05..=0.95).contains(&l.split_ratio) { l.split_ratio } else { default.split_ratio },
        scale: if SCALE_OPTIONS.contains(&l.scale.as_str()) { l.scale } else { default.scale },
        enabled_addons,
        max_combinations: if l.max_combinations >= 1 { l.max_combinations } else { default.max_combinations },
        language: if LANGUAGE_OPTIONS.contains(&l.language.as_str()) { l.language } else { default.language },
    }
}
```

Добавьте `save_language` сразу после `save_scale` (в самом конце файла):

```rust
/// Сохраняет выбранный язык, не трогая остальную раскладку — по той же
/// причине, что и save_scale/save_addons.
pub fn save_language(language: &str) {
    let mut current = load();
    current.language = language.to_string();
    save(&current);
}
```

- [ ] **Step 3: `commands.rs` — новые команды**

Добавьте сразу после `save_scale` (перед `save_addons`):

```rust
#[tauri::command]
pub fn save_language(language: String) {
    layout::save_language(&language);
}
```

Добавьте сразу после `is_user_added_component` (перед `#[tauri::command]
pub fn insert_component`):

```rust
#[tauri::command]
pub fn has_user_added_components(state: State<AppState>) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::has_user_added_components(&conn).map_err(map_err)
}
```

- [ ] **Step 4: `lib.rs` — зарегистрировать команды**

Замените:

```rust
            commands::component_exists,
            commands::is_user_added_component,
            commands::insert_component,
```

на:

```rust
            commands::component_exists,
            commands::is_user_added_component,
            commands::has_user_added_components,
            commands::insert_component,
```

Замените:

```rust
            commands::save_scale,
            commands::save_addons,
```

на:

```rust
            commands::save_scale,
            commands::save_language,
            commands::save_addons,
```

- [ ] **Step 5: Проверить компиляцию и тесты**

Run: `cd src-tauri && cargo check && cargo test --lib`
Expected: `cargo check` — без ошибок; `cargo test --lib` — все 23 теста
`ok` (layout.rs/commands.rs в этом проекте не покрыты тестами — см.
прецедент, ничего нового здесь не добавляется, это соответствует уже
сложившейся практике).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/layout.rs src-tauri/src/commands.rs src-tauri/src/lib.rs
git commit -m "feat: add Layout.language + save_language/has_user_added_components commands (Task 2 of B3+B4)"
```

---

### Task 3: `src/lib/api.ts`

**Files:**
- Modify: `src/lib/api.ts`

**Interfaces:**
- Consumes: `save_language`, `has_user_added_components` команды (Task 2).
- Produces (используется в Task 4/5/6):
  - `Layout.language: string`
  - `api.saveLanguage: (language: string) => Promise<void>`
  - `api.hasUserAddedComponents: () => Promise<boolean>`
  - `CURRENT_LANG` — **удаляется полностью**.

- [ ] **Step 1: Добавить `language` в `Layout`**

Замените:

```typescript
export interface Layout {
  side_panel_width: number;
  split_ratio: number;
  scale: string;
  enabled_addons: AddonId[];
  max_combinations: number;
}
```

на:

```typescript
export interface Layout {
  side_panel_width: number;
  split_ratio: number;
  scale: string;
  enabled_addons: AddonId[];
  max_combinations: number;
  language: string;
}
```

- [ ] **Step 2: Удалить `CURRENT_LANG`, добавить `saveLanguage`/`hasUserAddedComponents`**

Замените:

```typescript
export type FilterKind = "" | "Улучшение" | "Яд";

// Реальное переключение языка — план B3. До тех пор каждый вызов, которому
// нужен lang, использует эту константу — единая точка замены, когда язык
// станет настоящим state вместо константы.
export const CURRENT_LANG = "ru";
```

на:

```typescript
export type FilterKind = "" | "Улучшение" | "Яд";
```

Замените:

```typescript
  saveScale: (scale: string) => invoke<void>("save_scale", { scale }),
  saveAddons: (addons: AddonId[]) => invoke<void>("save_addons", { addons }),
```

на:

```typescript
  saveScale: (scale: string) => invoke<void>("save_scale", { scale }),
  saveLanguage: (language: string) => invoke<void>("save_language", { language }),
  saveAddons: (addons: AddonId[]) => invoke<void>("save_addons", { addons }),
```

Замените:

```typescript
  isUserAddedComponent: (id: number) => invoke<boolean>("is_user_added_component", { id }),
```

на:

```typescript
  isUserAddedComponent: (id: number) => invoke<boolean>("is_user_added_component", { id }),
  hasUserAddedComponents: () => invoke<boolean>("has_user_added_components"),
```

- [ ] **Step 3: Проверить компиляцию**

Run: `npx tsc -b`
Expected: FAIL — ошибки везде, где использовался `CURRENT_LANG`
(`App.tsx`, `EditorModal.tsx`) — "CURRENT_LANG is not exported". Ожидаемо,
чинится в Task 5/6.

- [ ] **Step 4: Commit**

```bash
git add src/lib/api.ts
git commit -m "feat: add Layout.language, saveLanguage/hasUserAddedComponents, remove CURRENT_LANG (Task 3 of B3+B4)"
```

---

### Task 4: `src/lib/languages.ts` (новый) + `src/components/SettingsModal.tsx`

**Files:**
- Create: `src/lib/languages.ts`
- Modify: `src/components/SettingsModal.tsx`

**Interfaces:**
- Consumes: `api.saveLanguage`, `api.hasUserAddedComponents` (Task 3).
- Produces (используется в Task 5, `EditorModal.tsx`, и в Task 6, `App.tsx`):
  - `export interface LanguageOption { value: string; label: string }`
  - `export const LANGUAGE_OPTIONS: LanguageOption[]`
  - `SettingsModal`'s новые пропы: `currentLanguage: string`,
    `onLanguageChange: (lang: string) => void`

- [ ] **Step 1: Создать `src/lib/languages.ts`**

```typescript
// languages.ts — официальные локализации Skyrim Special Edition (+
// китайский, только текстовый перевод, без озвучки) — общий словарь для
// SettingsModal (выбор языка) и EditorModal (подпись в диалоге "Новый
// ингредиент"). value должен совпадать с кодами, которые понимает бэкенд
// (component_translations.lang и т.д.) — zh-Hant, не zh (см.
// docs/superpowers/specs/2026-08-10-i18n-storage-design.md).

export interface LanguageOption {
  value: string;
  label: string;
}

export const LANGUAGE_OPTIONS: LanguageOption[] = [
  { value: "ru", label: "🇷🇺 Русский" },
  { value: "en", label: "🇬🇧 English" },
  { value: "fr", label: "🇫🇷 Français" },
  { value: "de", label: "🇩🇪 Deutsch" },
  { value: "it", label: "🇮🇹 Italiano" },
  { value: "es", label: "🇪🇸 Español" },
  { value: "pl", label: "🇵🇱 Polski" },
  { value: "ja", label: "🇯🇵 日本語" },
  { value: "zh-Hant", label: "🇨🇳 中文" },
];
```

- [ ] **Step 2: `SettingsModal.tsx` — убрать локальный `LANGUAGE_OPTIONS`, импортировать из `lib/`**

Замените:

```typescript
// Официальные локализации Skyrim (~8) + китайский (только текстовый перевод).
const LANGUAGE_OPTIONS = [
  { value: "ru", label: "🇷🇺 Русский" },
  { value: "en", label: "🇬🇧 English" },
  { value: "fr", label: "🇫🇷 Français" },
  { value: "de", label: "🇩🇪 Deutsch" },
  { value: "it", label: "🇮🇹 Italiano" },
  { value: "es", label: "🇪🇸 Español" },
  { value: "pl", label: "🇵🇱 Polski" },
  { value: "ja", label: "🇯🇵 日本語" },
  { value: "zh-Hant", label: "🇨🇳 中文" },
];
```

на (удаляется целиком — импорт добавляется в Step 3).

- [ ] **Step 3: `SettingsModal.tsx` — новые пропы, импорты, state**

Замените:

```typescript
import { api } from "../lib/api";
import {
  BASE_WINDOW_HEIGHT,
  BASE_WINDOW_WIDTH,
  MIN_WINDOW_HEIGHT,
  SCALE_FACTOR_BY_NAME,
} from "../lib/appTheme";
import type { AppScaleName, AppThemeName } from "../lib/appTheme";
import { ADDON_CHECKBOX_IDS, ADDON_LABELS } from "../lib/addons";
import type { AddonId } from "../lib/addons";

interface Props {
  opened: boolean;
  onClose: () => void;
  appTheme: AppThemeName;
  onAppThemeChange: (theme: AppThemeName) => void;
  scale: AppScaleName;
  onScaleChange: (scale: AppScaleName) => void;
  enabledAddons: AddonId[];
  onEnabledAddonsChange: (addons: AddonId[]) => void;
  maxCombinations: number;
  onMaxCombinationsChange: (maxCombinations: number) => void;
}
```

на:

```typescript
import { api } from "../lib/api";
import {
  BASE_WINDOW_HEIGHT,
  BASE_WINDOW_WIDTH,
  MIN_WINDOW_HEIGHT,
  SCALE_FACTOR_BY_NAME,
} from "../lib/appTheme";
import type { AppScaleName, AppThemeName } from "../lib/appTheme";
import { ADDON_CHECKBOX_IDS, ADDON_LABELS } from "../lib/addons";
import type { AddonId } from "../lib/addons";
import { LANGUAGE_OPTIONS } from "../lib/languages";

interface Props {
  opened: boolean;
  onClose: () => void;
  appTheme: AppThemeName;
  onAppThemeChange: (theme: AppThemeName) => void;
  scale: AppScaleName;
  onScaleChange: (scale: AppScaleName) => void;
  currentLanguage: string;
  onLanguageChange: (lang: string) => void;
  enabledAddons: AddonId[];
  onEnabledAddonsChange: (addons: AddonId[]) => void;
  maxCombinations: number;
  onMaxCombinationsChange: (maxCombinations: number) => void;
}
```

Замените:

```typescript
export function SettingsModal({
  opened,
  onClose,
  appTheme,
  onAppThemeChange,
  scale: appScale,
  onScaleChange,
  enabledAddons,
  onEnabledAddonsChange,
  maxCombinations,
  onMaxCombinationsChange,
}: Props) {
  const { colorScheme, setColorScheme } = useMantineColorScheme();

  const [scale, setScale] = useState<AppScaleName>(appScale);
  const [language, setLanguage] = useState<string>("ru");
  const [theme, setTheme] = useState<string>(
    appTheme === "skyrim" ? SKYRIM_LABEL : THEME_LABEL_BY_SCHEME[colorScheme],
  );
  const [checkedAddons, setCheckedAddons] = useState<AddonId[]>(enabledAddons);
  // Текстовое поле — храним как строку (ограничена только цифрами через
  // onChange), а не число, чтобы не мешать вводу (например, стереть всё и
  // напечатать заново).
  const [maxCombinationsInput, setMaxCombinationsInput] = useState(String(maxCombinations));

  // При каждом открытии окна — подхватить реально применённые сейчас тему,
  // масштаб и набор дополнений (а не то, что было выбрано в списке, но не
  // применено).
  useEffect(() => {
    if (opened) {
      setTheme(appTheme === "skyrim" ? SKYRIM_LABEL : THEME_LABEL_BY_SCHEME[colorScheme]);
      setScale(appScale);
      setCheckedAddons(enabledAddons);
      setMaxCombinationsInput(String(maxCombinations));
    }
  }, [opened, appTheme, colorScheme, appScale, enabledAddons, maxCombinations]);
```

на:

```typescript
export function SettingsModal({
  opened,
  onClose,
  appTheme,
  onAppThemeChange,
  scale: appScale,
  onScaleChange,
  currentLanguage,
  onLanguageChange,
  enabledAddons,
  onEnabledAddonsChange,
  maxCombinations,
  onMaxCombinationsChange,
}: Props) {
  const { colorScheme, setColorScheme } = useMantineColorScheme();

  const [scale, setScale] = useState<AppScaleName>(appScale);
  const [language, setLanguage] = useState<string>(currentLanguage);
  const [theme, setTheme] = useState<string>(
    appTheme === "skyrim" ? SKYRIM_LABEL : THEME_LABEL_BY_SCHEME[colorScheme],
  );
  const [checkedAddons, setCheckedAddons] = useState<AddonId[]>(enabledAddons);
  // Текстовое поле — храним как строку (ограничена только цифрами через
  // onChange), а не число, чтобы не мешать вводу (например, стереть всё и
  // напечатать заново).
  const [maxCombinationsInput, setMaxCombinationsInput] = useState(String(maxCombinations));
  const [languageWarningOpen, setLanguageWarningOpen] = useState(false);

  // При каждом открытии окна — подхватить реально применённые сейчас тему,
  // масштаб, язык и набор дополнений (а не то, что было выбрано в списке,
  // но не применено).
  useEffect(() => {
    if (opened) {
      setTheme(appTheme === "skyrim" ? SKYRIM_LABEL : THEME_LABEL_BY_SCHEME[colorScheme]);
      setScale(appScale);
      setLanguage(currentLanguage);
      setCheckedAddons(enabledAddons);
      setMaxCombinationsInput(String(maxCombinations));
    }
  }, [opened, appTheme, colorScheme, appScale, currentLanguage, enabledAddons, maxCombinations]);
```

- [ ] **Step 4: `SettingsModal.tsx` — разделить `handleApply` на проверку языка + `applyAll`**

Замените:

```typescript
  function handleApply() {
    if (theme === SKYRIM_LABEL) {
      onAppThemeChange("skyrim");
    } else {
      onAppThemeChange("default");
      setColorScheme(THEME_SCHEME_BY_LABEL[theme] ?? "auto");
    }

    // Ресайзим окно, только если масштаб реально поменяли — иначе
    // "Применить" ради темы/языка/дополнений насильно возвращало бы окно к
    // дефолтному для текущего масштаба размеру, затирая ручной ресайз
    // пользователя (см. баг: окно "прыгало" при любом Apply).
    if (scale !== appScale) {
      const factor = SCALE_FACTOR_BY_NAME[scale];
      const height = Math.max(BASE_WINDOW_HEIGHT * factor, MIN_WINDOW_HEIGHT);
      getCurrentWindow().setSize(new LogicalSize(BASE_WINDOW_WIDTH * factor, height));
      api.saveScale(scale);
    }
    onScaleChange(scale);

    onEnabledAddonsChange(checkedAddons);
    api.saveAddons(checkedAddons);

    const parsedMax = parseInt(maxCombinationsInput, 10);
    const nextMax = Number.isFinite(parsedMax) && parsedMax >= 1 ? parsedMax : DEFAULT_MAX_COMBINATIONS;
    onMaxCombinationsChange(nextMax);
    api.saveMaxCombinations(nextMax);

    onClose();
  }
```

на:

```typescript
  // Смена языка — единственная настройка, которая может скрыть уже
  // существующие данные пользователя (его собственные ингредиенты, видимые
  // только на языке создания, см. design doc, раздел B3+B4). Если язык
  // реально меняется и у пользователя есть хотя бы один такой ингредиент —
  // сначала спрашиваем подтверждение, весь остальной Apply откладывается до
  // ответа.
  function handleApply() {
    if (language !== currentLanguage) {
      api.hasUserAddedComponents().then((has) => {
        if (has) {
          setLanguageWarningOpen(true);
        } else {
          applyAll();
        }
      });
      return;
    }
    applyAll();
  }

  function applyAll() {
    if (theme === SKYRIM_LABEL) {
      onAppThemeChange("skyrim");
    } else {
      onAppThemeChange("default");
      setColorScheme(THEME_SCHEME_BY_LABEL[theme] ?? "auto");
    }

    // Ресайзим окно, только если масштаб реально поменяли — иначе
    // "Применить" ради темы/языка/дополнений насильно возвращало бы окно к
    // дефолтному для текущего масштаба размеру, затирая ручной ресайз
    // пользователя (см. баг: окно "прыгало" при любом Apply).
    if (scale !== appScale) {
      const factor = SCALE_FACTOR_BY_NAME[scale];
      const height = Math.max(BASE_WINDOW_HEIGHT * factor, MIN_WINDOW_HEIGHT);
      getCurrentWindow().setSize(new LogicalSize(BASE_WINDOW_WIDTH * factor, height));
      api.saveScale(scale);
    }
    onScaleChange(scale);

    if (language !== currentLanguage) {
      onLanguageChange(language);
      api.saveLanguage(language);
    }

    onEnabledAddonsChange(checkedAddons);
    api.saveAddons(checkedAddons);

    const parsedMax = parseInt(maxCombinationsInput, 10);
    const nextMax = Number.isFinite(parsedMax) && parsedMax >= 1 ? parsedMax : DEFAULT_MAX_COMBINATIONS;
    onMaxCombinationsChange(nextMax);
    api.saveMaxCombinations(nextMax);

    onClose();
  }
```

- [ ] **Step 5: `SettingsModal.tsx` — обновить Select "Язык" и добавить модалку подтверждения**

Замените:

```typescript
              <Select
                label="Язык"
                data={LANGUAGE_OPTIONS}
                value={language}
                onChange={(v) => setLanguage(v ?? language)}
                allowDeselect={false}
                comboboxProps={{ withinPortal: true }}
              />
```

на (без изменений по сути — `LANGUAGE_OPTIONS` теперь импортируется, не
объявляется локально; JSX не меняется, оставлен для ориентира — реально
менять тут нечего, просто убедитесь, что Select ссылается на импортированный
`LANGUAGE_OPTIONS`).

Добавьте новую модалку сразу перед закрывающим `</Modal>` самого
`SettingsModal` (после блока "Нижняя панель — Применить / Отмена", то есть
после `</Group>`, перед последним `</div></Modal>`):

```typescript
      <Modal
        opened={languageWarningOpen}
        onClose={() => setLanguageWarningOpen(false)}
        title="Подтверждение"
        size="sm"
      >
        <Text size="sm" mb="md">
          У вас есть ингредиенты, добавленные вручную — они видны только на
          языке, на котором были созданы. После смены языка они пропадут из
          списков (ничего не удаляется — снова появятся, если вернуться на
          прежний язык). Продолжить?
        </Text>
        <Group justify="flex-end">
          <Button variant="default" onClick={() => setLanguageWarningOpen(false)}>
            Отмена
          </Button>
          <Button
            onClick={() => {
              setLanguageWarningOpen(false);
              applyAll();
            }}
          >
            Продолжить
          </Button>
        </Group>
      </Modal>
```

(`Group` уже импортирован из `@mantine/core` в этом файле — использован в
"Нижней панели"; `Text`/`Button`/`Modal` тоже уже импортированы.)

- [ ] **Step 6: Проверить компиляцию**

Run: `npx tsc -b`
Expected: FAIL — ошибки в `App.tsx` (передаёт `SettingsModal` пропы старой
формы, без `currentLanguage`/`onLanguageChange`) и всё ещё в
`EditorModal.tsx`. Ошибок внутри `SettingsModal.tsx`/`languages.ts` быть не
должно.

- [ ] **Step 7: Commit**

```bash
git add src/lib/languages.ts src/components/SettingsModal.tsx
git commit -m "feat: wire SettingsModal to real language state + Apply confirmation (Task 4 of B3+B4)"
```

---

### Task 5: `src/components/EditorModal.tsx`

**Files:**
- Modify: `src/components/EditorModal.tsx`

**Interfaces:**
- Consumes: `LANGUAGE_OPTIONS` из `../lib/languages` (Task 4).
- Produces (используется в Task 6, `App.tsx`):
  - `EditorModal`'s новый проп `lang: string` (заменяет внутренний
    `CURRENT_LANG`).

- [ ] **Step 1: Заменить импорт `CURRENT_LANG` на проп `lang`, добавить импорт `LANGUAGE_OPTIONS`**

Замените:

```typescript
import { api, CURRENT_LANG } from "../lib/api";
import type { ComponentNameInfo, PropertyInfo } from "../lib/api";
import { GOOD_QUALITY_SIZE, useAdaptiveImageSize } from "../lib/useAdaptiveImageSize";

interface Props {
  opened: boolean;
  onClose: () => void;
  onChanged: () => void; // список компонентов на главном экране мог измениться
}
```

на:

```typescript
import { api } from "../lib/api";
import type { ComponentNameInfo, PropertyInfo } from "../lib/api";
import { GOOD_QUALITY_SIZE, useAdaptiveImageSize } from "../lib/useAdaptiveImageSize";
import { LANGUAGE_OPTIONS } from "../lib/languages";

interface Props {
  opened: boolean;
  onClose: () => void;
  onChanged: () => void; // список компонентов на главном экране мог измениться
  lang: string;
}
```

Замените:

```typescript
export function EditorModal({ opened, onClose, onChanged }: Props) {
```

на:

```typescript
export function EditorModal({ opened, onClose, onChanged, lang }: Props) {
```

- [ ] **Step 2: Заменить все использования `CURRENT_LANG` на `lang`**

Замените (ровно эти 9 вхождений — `useEffect`, `loadComponent`,
`confirmNewName`, `performSave` ×2, `handleDelete`):

```typescript
    api.getProperties(CURRENT_LANG).then(setAllProperties).catch(() => {});
    api
      .getComponentNames(CURRENT_LANG)
```

на:

```typescript
    api.getProperties(lang).then(setAllProperties).catch(() => {});
    api
      .getComponentNames(lang)
```

Замените:

```typescript
    const [props, media, userAdded] = await Promise.all([
      api.getComponentPropertiesWithTypes(id, CURRENT_LANG),
      api.getComponentMedia(id, CURRENT_LANG),
      api.isUserAddedComponent(id),
    ]);
```

на:

```typescript
    const [props, media, userAdded] = await Promise.all([
      api.getComponentPropertiesWithTypes(id, lang),
      api.getComponentMedia(id, lang),
      api.isUserAddedComponent(id),
    ]);
```

Замените:

```typescript
    if (await api.componentExists(name, CURRENT_LANG)) {
```

на:

```typescript
    if (await api.componentExists(name, lang)) {
```

Замените:

```typescript
      let id = loadedId;
      if (isNew) {
        id = await api.insertComponent(loadedName, CURRENT_LANG, propIds);
        const ns = await api.getComponentNames(CURRENT_LANG);
        setNames(ns);
      } else if (editable && id !== null) {
        await api.updateComponentProperties(id, propIds);
      }
      if (id !== null) {
        await api.setComponentMedia(id, CURRENT_LANG, imageBase64, descriptionValue);
      }
```

на:

```typescript
      let id = loadedId;
      if (isNew) {
        id = await api.insertComponent(loadedName, lang, propIds);
        const ns = await api.getComponentNames(lang);
        setNames(ns);
      } else if (editable && id !== null) {
        await api.updateComponentProperties(id, propIds);
      }
      if (id !== null) {
        await api.setComponentMedia(id, lang, imageBase64, descriptionValue);
      }
```

Замените:

```typescript
      await api.deleteComponent(loadedId);
      const ns = await api.getComponentNames(CURRENT_LANG);
```

на:

```typescript
      await api.deleteComponent(loadedId);
      const ns = await api.getComponentNames(lang);
```

- [ ] **Step 3: Подпись у "Описание"**

Замените:

```typescript
        <Textarea
          key={loadedName}
          label="Описание"
          styles={{ input: { minHeight: DESCRIPTION_HEIGHT } }}
          ref={descriptionRef}
          defaultValue={description}
          onChange={() => {
            setDescriptionTouched(true);
            setDirty(true);
          }}
        />
```

на:

```typescript
        <div>
          <Group gap={6} align="baseline">
            <Text size="sm" fw={700}>
              Описание
            </Text>
            <Text size="xs" c="dimmed">
              (Только для текущего языка)
            </Text>
          </Group>
          <Textarea
            key={loadedName}
            styles={{ input: { minHeight: DESCRIPTION_HEIGHT } }}
            ref={descriptionRef}
            defaultValue={description}
            onChange={() => {
              setDescriptionTouched(true);
              setDirty(true);
            }}
          />
        </div>
```

(Убрали `label="Описание"` с самого `Textarea` — метка теперь рисуется
отдельным `Text` над полем, чтобы рядом влезла пояснительная подпись;
визуально должно выглядеть так же, как остальные подписанные поля в этой
форме, см. "Изображение" чуть выше.)

- [ ] **Step 4: Строка с языком в диалоге "Новый ингредиент"**

Замените:

```typescript
      <Modal opened={newDialogOpen} onClose={() => setNewDialogOpen(false)} title="Новый ингредиент" size="sm">
        <TextInput
          key={newDialogGeneration}
          label="Название"
          ref={newNameRef}
          defaultValue=""
          data-autofocus
        />
        <Group justify="flex-end" mt="md">
          <Button variant="default" onClick={() => setNewDialogOpen(false)}>
            Отмена
          </Button>
          <Button onClick={confirmNewName}>Создать</Button>
        </Group>
      </Modal>
```

на:

```typescript
      <Modal opened={newDialogOpen} onClose={() => setNewDialogOpen(false)} title="Новый ингредиент" size="sm">
        <TextInput
          key={newDialogGeneration}
          label="Название"
          ref={newNameRef}
          defaultValue=""
          data-autofocus
        />
        <Text size="xs" c="dimmed" mt={4}>
          Ингредиент будет виден только при выбранном языке —{" "}
          {LANGUAGE_OPTIONS.find((l) => l.value === lang)?.label ?? lang}.
        </Text>
        <Group justify="flex-end" mt="md">
          <Button variant="default" onClick={() => setNewDialogOpen(false)}>
            Отмена
          </Button>
          <Button onClick={confirmNewName}>Создать</Button>
        </Group>
      </Modal>
```

- [ ] **Step 5: Проверить компиляцию**

Run: `npx tsc -b`
Expected: FAIL — ошибки только в `App.tsx` (не передаёт `EditorModal` новый
проп `lang`, всё ещё импортирует `CURRENT_LANG`). Ошибок внутри
`EditorModal.tsx` быть не должно.

- [ ] **Step 6: Commit**

```bash
git add src/components/EditorModal.tsx
git commit -m "feat: switch EditorModal.tsx to lang prop + language hints (Task 5 of B3+B4)"
```

---

### Task 6: `src/App.tsx`

**Files:**
- Modify: `src/App.tsx`

**Interfaces:**
- Consumes: обновлённые `api.ts` (Task 3), `SettingsModal`'s
  `currentLanguage`/`onLanguageChange` (Task 4), `EditorModal`'s `lang`
  (Task 5).
- Produces: ничего, что потребляют другие задачи этого плана — последняя
  задача, после неё весь проект должен собираться.

- [ ] **Step 1: Заменить импорт `CURRENT_LANG` на state `language`**

Замените:

```typescript
import { api, CURRENT_LANG } from "./lib/api";
import type { ComponentNameInfo, CombinationResult, FilterKind, PropertyInfo } from "./lib/api";
```

на:

```typescript
import { api } from "./lib/api";
import type { ComponentNameInfo, CombinationResult, FilterKind, PropertyInfo } from "./lib/api";
```

Замените:

```typescript
const HANDLE_SIZE = 8;
const DEFAULT_MAX_COMBINATIONS = 100;
```

на:

```typescript
const HANDLE_SIZE = 8;
const DEFAULT_MAX_COMBINATIONS = 100;
const DEFAULT_LANGUAGE = "ru";
```

Замените:

```typescript
  // Ограничение числа результатов "Парных"/"Тройных сочетаний" (см.
  // SettingsModal). Дефолт — пока сохранённые настройки не подгрузились.
  const [maxCombinations, setMaxCombinations] = useState(DEFAULT_MAX_COMBINATIONS);
```

на:

```typescript
  // Ограничение числа результатов "Парных"/"Тройных сочетаний" (см.
  // SettingsModal). Дефолт — пока сохранённые настройки не подгрузились.
  const [maxCombinations, setMaxCombinations] = useState(DEFAULT_MAX_COMBINATIONS);
  // Текущий язык (план B3) — раньше был захардкоженной константой CURRENT_LANG.
  const [language, setLanguage] = useState(DEFAULT_LANGUAGE);
```

- [ ] **Step 2: Слить загрузку `properties`/`componentNames` в один эффект, реагирующий и на `enabledAddons`, и на `language`**

Замените:

```typescript
  // --- Начальная загрузка: список свойств, сохранённая раскладка (список
  // компонентов, ограниченный дополнениями, подтягивается отдельным
  // эффектом ниже — он же сработает повторно, когда сохранённый набор
  // дополнений подгрузится из getLayout и отличается от дефолтного) ---
  useEffect(() => {
    api.getProperties(CURRENT_LANG).then(setProperties);
    api.getLayout().then((l) => {
      setSidePanelWidth(l.side_panel_width);
      setSplitRatio(l.split_ratio);
      setEnabledAddons(l.enabled_addons);
      setMaxCombinations(l.max_combinations);
    });
  }, []);
```

на:

```typescript
  // --- Начальная загрузка: сохранённая раскладка (список свойств и список
  // компонентов, ограниченный дополнениями и языком, подтягиваются отдельным
  // эффектом ниже — он же сработает повторно, когда сохранённые дополнения
  // и/или язык подгрузятся из getLayout и отличаются от дефолтных) ---
  useEffect(() => {
    api.getLayout().then((l) => {
      setSidePanelWidth(l.side_panel_width);
      setSplitRatio(l.split_ratio);
      setEnabledAddons(l.enabled_addons);
      setMaxCombinations(l.max_combinations);
      setLanguage(l.language);
    });
  }, []);
```

Замените:

```typescript
  // --- Список ингредиентов для выбора компонента, ограниченный включёнными
  // дополнениями. Срабатывает и при первой загрузке (с дефолтным "все
  // включены"), и при каждом применении новых настроек в SettingsModal.
  // Прежний выбранный компонент и результаты предыдущего поиска могли
  // ссылаться на теперь скрытые ингредиенты — сбрасываем оба, чтобы не
  // показывать стухшие данные; пользователь просто ищет заново.
  useEffect(() => {
    api.getComponentNamesFiltered(enabledAddons, CURRENT_LANG).then((names) => {
      setComponentNames(names);
      setComponentSelect((prev) => (prev !== null && names.some((n) => n.id === prev) ? prev : null));
    });
    lastCombosRef.current = [];
    setEnabledComponents({});
    setTopMode({ kind: "empty" });
    setBottomMode({ kind: "list" });
    setResults([]);
    setResultsHeader("Найдено 0 комбинаций");
  }, [enabledAddons]);

  function refreshLists() {
    api.getProperties(CURRENT_LANG).then(setProperties);
    api.getComponentNamesFiltered(enabledAddons, CURRENT_LANG).then(setComponentNames);
  }
```

на:

```typescript
  // --- Список свойств и список ингредиентов, ограниченный включёнными
  // дополнениями и текущим языком. Срабатывает при первой загрузке (с
  // дефолтными значениями), при каждом применении новых настроек в
  // SettingsModal (дополнения ИЛИ язык) и при смене языка через Настройки.
  // Прежний выбранный компонент и результаты предыдущего поиска могли
  // ссылаться на теперь скрытые ингредиенты (другой набор дополнений или
  // язык, в котором их вообще нет — см. design doc про пользовательские
  // ингредиенты) — сбрасываем оба, чтобы не показывать стухшие данные;
  // пользователь просто ищет заново.
  useEffect(() => {
    api.getProperties(language).then(setProperties);
    api.getComponentNamesFiltered(enabledAddons, language).then((names) => {
      setComponentNames(names);
      setComponentSelect((prev) => (prev !== null && names.some((n) => n.id === prev) ? prev : null));
    });
    lastCombosRef.current = [];
    setEnabledComponents({});
    setTopMode({ kind: "empty" });
    setBottomMode({ kind: "list" });
    setResults([]);
    setResultsHeader("Найдено 0 комбинаций");
  }, [enabledAddons, language]);

  function refreshLists() {
    api.getProperties(language).then(setProperties);
    api.getComponentNamesFiltered(enabledAddons, language).then(setComponentNames);
  }
```

- [ ] **Step 3: Заменить оставшиеся использования `CURRENT_LANG` в обработчиках поиска**

Замените:

```typescript
      const found = await api.findCombinations(chosen, filter, enabledAddons, CURRENT_LANG);
```

на:

```typescript
      const found = await api.findCombinations(chosen, filter, enabledAddons, language);
```

Замените:

```typescript
      const props = await api.getComponentProperties(componentSelect, CURRENT_LANG);
      const selectedName = componentNames.find((c) => c.id === componentSelect)?.name ?? "";
      lastCombosRef.current = [];
      setEnabledComponents({});
      setTopMode({ kind: "properties", props });
      setResultsHeader(selectedName);

      const media = await api.getComponentMedia(componentSelect, CURRENT_LANG);
```

на:

```typescript
      const props = await api.getComponentProperties(componentSelect, language);
      const selectedName = componentNames.find((c) => c.id === componentSelect)?.name ?? "";
      lastCombosRef.current = [];
      setEnabledComponents({});
      setTopMode({ kind: "properties", props });
      setResultsHeader(selectedName);

      const media = await api.getComponentMedia(componentSelect, language);
```

Замените:

```typescript
      const found = await api.findPairs(filter, enabledAddons, maxCombinations, CURRENT_LANG);
```

на:

```typescript
      const found = await api.findPairs(filter, enabledAddons, maxCombinations, language);
```

Замените:

```typescript
      const found = await api.findMaxCombinations(filter, enabledAddons, maxCombinations, CURRENT_LANG);
```

на:

```typescript
      const found = await api.findMaxCombinations(filter, enabledAddons, maxCombinations, language);
```

- [ ] **Step 4: Пробросить `lang`/`currentLanguage`/`onLanguageChange` в `EditorModal`/`SettingsModal`**

Замените:

```typescript
      <EditorModal
        opened={editorOpen}
        onClose={() => setEditorOpen(false)}
        onChanged={() => {
          refreshLists();
        }}
      />
```

на:

```typescript
      <EditorModal
        opened={editorOpen}
        onClose={() => setEditorOpen(false)}
        onChanged={() => {
          refreshLists();
        }}
        lang={language}
      />
```

Замените:

```typescript
      <SettingsModal
        opened={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        appTheme={appTheme}
        onAppThemeChange={onAppThemeChange}
        scale={scale}
        onScaleChange={onScaleChange}
        enabledAddons={enabledAddons}
        onEnabledAddonsChange={setEnabledAddons}
        maxCombinations={maxCombinations}
        onMaxCombinationsChange={setMaxCombinations}
      />
```

на:

```typescript
      <SettingsModal
        opened={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        appTheme={appTheme}
        onAppThemeChange={onAppThemeChange}
        scale={scale}
        onScaleChange={onScaleChange}
        currentLanguage={language}
        onLanguageChange={setLanguage}
        enabledAddons={enabledAddons}
        onEnabledAddonsChange={setEnabledAddons}
        maxCombinations={maxCombinations}
        onMaxCombinationsChange={setMaxCombinations}
      />
```

- [ ] **Step 5: Проверить компиляцию — весь проект должен быть зелёным**

Run: `npx tsc -b`
Expected: PASS, без ошибок во всём проекте.

Run: `npm run build`
Expected: PASS.

Run: `npm run lint`
Expected: PASS (новых предупреждений от этого плана быть не должно).

- [ ] **Step 6: Commit**

```bash
git add src/App.tsx
git commit -m "feat: wire real language state through App.tsx (Task 6 of B3+B4)"
```

---

## После выполнения плана

Переключение языка в Настройках реально работает: список ингредиентов,
свойства, результаты поиска и редактор базы — всё читается на выбранном
языке; при смене языка с существующими пользовательскими ингредиентами
показывается предупреждение перед применением. Текст интерфейса
по-прежнему целиком на русском (отдельная будущая задача). Кросс-языковая
видимость пользовательских ингредиентов (вариант (a) из design doc) — тоже
отдельная будущая задача, не начата.

Перед тем как считать план полностью готовым — попросить пользователя
запустить `npm run tauri dev` самостоятельно и подтвердить: переключение
языка в Настройках меняет отображаемые названия, предупреждение появляется
только при наличии пользовательских ингредиентов, диалог "Новый ингредиент"
показывает актуальный язык. Эта задача сознательно не поднимает dev-сервер
и не пытается автоматизировать браузер (см. правило проекта про визуальную
проверку).
