// db.rs — работа со встроенной SQLite-базой: схема, инициализация тестовыми
// данными, CRUD компонентов и поиск сочетаний/пар. Прямой аналог db.go из
// Go-версии, перенесённый на rusqlite.

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::addons::Addon;
use crate::creations::{
    CREATION_DESCRIPTIONS, CREATION_IMAGES, CREATION_INGREDIENTS, CREATION_PROPERTIES,
    FISHING_INGREDIENTS, PLAGUE_INGREDIENTS, SAINTS_INGREDIENTS,
};
use crate::rare_curios::{RARE_CURIOS_DESCRIPTION, RARE_CURIOS_IMAGES, RARE_CURIOS_INGREDIENTS, RARE_CURIOS_PROPERTIES};
use crate::seed_data::{DAWNGUARD_INGREDIENTS, DRAGONBORN_INGREDIENTS, HEARTHFIRE_INGREDIENTS, INGREDIENTS, MEDIA, PROPERTIES};

pub const TYPE_BENEFIT: &str = "Улучшение";
pub const TYPE_POISON: &str = "Яд";

/// Открывает базу, а если файла ещё нет — создаёт схему и наполняет тестовыми
/// данными. Затем всегда прогоняет миграции (медиа-колонки, ингредиенты
/// Rare Curios) — аналог ensureDB.
pub fn ensure_db(path: &Path) -> rusqlite::Result<Connection> {
    let need_init = !path.exists();
    let conn = Connection::open(path)?;

    if need_init {
        init_db(&conn)?;
    }

    migrate_media(&conn)?;
    migrate_rare_curios(&conn)?;
    migrate_creations(&conn)?;
    migrate_addon_ids(&conn)?;
    migrate_images_to_blob(&conn)?;
    Ok(conn)
}

fn init_db(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE properties (
            id   INTEGER PRIMARY KEY,
            name TEXT UNIQUE NOT NULL,
            type TEXT NOT NULL
        );
        CREATE TABLE components (
            id          INTEGER PRIMARY KEY,
            name        TEXT NOT NULL,
            image_url   TEXT NOT NULL DEFAULT '',
            description TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE component_properties (
            component_id INTEGER NOT NULL,
            property_id  INTEGER NOT NULL
        );",
    )?;

    let mut prop_id: HashMap<&str, i64> = HashMap::new();
    for (name, typ) in PROPERTIES {
        conn.execute(
            "INSERT INTO properties (name, type) VALUES (?1, ?2)",
            params![name, typ],
        )?;
        prop_id.insert(name, conn.last_insert_rowid());
    }

    let media_map: HashMap<&str, (&str, &str)> = MEDIA
        .iter()
        .map(|(name, url, desc)| (*name, (*url, *desc)))
        .collect();

    for (name, props) in INGREDIENTS {
        let (image_url, description) = media_map.get(name).copied().unwrap_or(("", ""));
        conn.execute(
            "INSERT INTO components (name, image_url, description) VALUES (?1, ?2, ?3)",
            params![name, image_url, description],
        )?;
        let component_id = conn.last_insert_rowid();
        for prop_name in props {
            if let Some(&pid) = prop_id.get(prop_name) {
                conn.execute(
                    "INSERT INTO component_properties (component_id, property_id) VALUES (?1, ?2)",
                    params![component_id, pid],
                )?;
            }
        }
    }

    Ok(())
}

/// Добавляет колонки image_url/description, если их ещё нет, и подтягивает
/// данные из MEDIA туда, где сейчас пусто (не затирая ручные правки).
/// image_url здесь используется только как "откуда взять файл" для
/// однократного переноса в БД (см. migrate_images_to_blob) — сама программа
/// картинки по этому пути больше не читает.
fn migrate_media(conn: &Connection) -> rusqlite::Result<()> {
    let mut has_image_url = false;
    let mut has_description = false;
    let mut has_image_blob = false;
    {
        let mut stmt = conn.prepare("PRAGMA table_info(components)")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            match name.as_str() {
                "image_url" => has_image_url = true,
                "description" => has_description = true,
                "image" => has_image_blob = true,
                _ => {}
            }
        }
    }

    if !has_image_url {
        conn.execute(
            "ALTER TABLE components ADD COLUMN image_url TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }
    if !has_description {
        conn.execute(
            "ALTER TABLE components ADD COLUMN description TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }
    if !has_image_blob {
        conn.execute("ALTER TABLE components ADD COLUMN image BLOB", [])?;
    }

    for (name, image_url, description) in MEDIA {
        conn.execute(
            "UPDATE components
             SET image_url = CASE WHEN image_url = '' THEN ?1 ELSE image_url END,
                 description = CASE WHEN description = '' THEN ?2 ELSE description END
             WHERE name = ?3",
            params![image_url, description, name],
        )?;
    }

    Ok(())
}

/// Переносит картинки из файлов (путь в image_url — из поставляемой вместе
/// со сборкой папки images/) прямо в БД, в колонку image (BLOB), — один раз
/// для каждого компонента, у которого блоба ещё нет. После этого папка
/// images/ программе больше не нужна: и старые данные, и всё, что добавит
/// пользователь через "Редактировать базу" (там теперь выбор файла через
/// системный диалог, см. editor.rs), хранится прямо внутри alchemist.db.
/// Ошибка чтения отдельного файла (например, если папку images/ забыли
/// скопировать) не прерывает всю миграцию — такой компонент просто
/// остаётся без картинки, как и раньше.
fn migrate_images_to_blob(conn: &Connection) -> rusqlite::Result<()> {
    let mut targets: Vec<(i64, String)> = Vec::new();
    {
        let mut stmt = conn.prepare(
            "SELECT id, image_url FROM components
             WHERE (image IS NULL OR length(image) = 0) AND image_url != ''",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            targets.push((row.get(0)?, row.get(1)?));
        }
    }

    for (id, image_url) in targets {
        if let Ok(bytes) = std::fs::read(&image_url) {
            conn.execute("UPDATE components SET image = ?1 WHERE id = ?2", params![bytes, id])?;
        }
    }

    Ok(())
}

/// Добавляет в базу компоненты дополнения Rare Curios (и недостающие для них
/// свойства), если их там ещё нет. Выполняется при каждом запуске — что для
/// новой базы, что для уже существующей (созданной до появления Rare Curios
/// в этом справочнике), поэтому у уже имеющихся пользователей ингредиенты
/// появятся сами по себе после обновления программы, без пересоздания базы.
/// Уже существующие компоненты (по имени) не трогаются — если кто-то успел
/// поправить их вручную через "Редактировать базу", правки не потеряются.
fn migrate_rare_curios(conn: &Connection) -> rusqlite::Result<()> {
    // 1) Добираем недостающие свойства.
    let mut prop_id: HashMap<String, i64> = HashMap::new();
    {
        let mut stmt = conn.prepare("SELECT id, name FROM properties")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            prop_id.insert(name, id);
        }
    }
    for (name, typ) in RARE_CURIOS_PROPERTIES {
        if !prop_id.contains_key(*name) {
            conn.execute(
                "INSERT INTO properties (name, type) VALUES (?1, ?2)",
                params![name, typ],
            )?;
            prop_id.insert((*name).to_string(), conn.last_insert_rowid());
        }
    }

    // 2) Добираем недостающие компоненты (по имени) вместе с их свойствами и медиа.
    let mut existing_names: HashSet<String> = HashSet::new();
    {
        let mut stmt = conn.prepare("SELECT name FROM components")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            existing_names.insert(row.get::<_, String>(0)?);
        }
    }

    let image_map: HashMap<&str, &str> = RARE_CURIOS_IMAGES.iter().copied().collect();

    for (name, props) in RARE_CURIOS_INGREDIENTS {
        if existing_names.contains(*name) {
            continue;
        }
        let image_url = image_map.get(name).copied().unwrap_or("");
        conn.execute(
            "INSERT INTO components (name, image_url, description) VALUES (?1, ?2, ?3)",
            params![name, image_url, RARE_CURIOS_DESCRIPTION],
        )?;
        let component_id = conn.last_insert_rowid();
        for prop_name in props {
            if let Some(&pid) = prop_id.get(*prop_name) {
                conn.execute(
                    "INSERT INTO component_properties (component_id, property_id) VALUES (?1, ?2)",
                    params![component_id, pid],
                )?;
            }
        }
    }

    Ok(())
}

/// Добавляет в базу ингредиенты творений "Рыбалка", "Святые и соблазнители"
/// и "Чума мертвецов" — по тому же принципу, что и migrate_rare_curios:
/// добирает недостающие свойства, затем добирает недостающие (по имени)
/// компоненты. Компоненты, которые уже есть в базе под тем же именем (в
/// частности, "Гнилая чешуйка" и "Огненный гриб" — они совпадают с Rare
/// Curios), не трогаются и не перезаписываются.
fn migrate_creations(conn: &Connection) -> rusqlite::Result<()> {
    let mut prop_id: HashMap<String, i64> = HashMap::new();
    {
        let mut stmt = conn.prepare("SELECT id, name FROM properties")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            prop_id.insert(name, id);
        }
    }
    for (name, typ) in CREATION_PROPERTIES {
        if !prop_id.contains_key(*name) {
            conn.execute(
                "INSERT INTO properties (name, type) VALUES (?1, ?2)",
                params![name, typ],
            )?;
            prop_id.insert((*name).to_string(), conn.last_insert_rowid());
        }
    }

    let mut existing_names: HashSet<String> = HashSet::new();
    {
        let mut stmt = conn.prepare("SELECT name FROM components")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            existing_names.insert(row.get::<_, String>(0)?);
        }
    }

    let image_map: HashMap<&str, &str> = CREATION_IMAGES.iter().copied().collect();
    let description_map: HashMap<&str, &str> = CREATION_DESCRIPTIONS.iter().copied().collect();

    for (name, props) in CREATION_INGREDIENTS {
        if existing_names.contains(*name) {
            continue;
        }
        let image_url = image_map.get(name).copied().unwrap_or("");
        let description = description_map.get(name).copied().unwrap_or("");
        conn.execute(
            "INSERT INTO components (name, image_url, description) VALUES (?1, ?2, ?3)",
            params![name, image_url, description],
        )?;
        let component_id = conn.last_insert_rowid();
        for prop_name in props {
            if let Some(&pid) = prop_id.get(*prop_name) {
                conn.execute(
                    "INSERT INTO component_properties (component_id, property_id) VALUES (?1, ?2)",
                    params![component_id, pid],
                )?;
            }
        }
        // Компонент только что вставлен — добавляем его в множество, чтобы
        // корректно обработать возможные повторы имени внутри самого
        // CREATION_INGREDIENTS (на случай ошибок в исходных данных).
        existing_names.insert((*name).to_string());
    }

    Ok(())
}

/// Проставляет components.addon (см. addons.rs) для всех известных по
/// имени ингредиентов. Выполняется при каждом запуске и, в отличие от
/// migrate_media, безусловно ПЕРЕЗАПИСЫВАЕТ значение по каждому известному
/// имени — так что если разметка когда-нибудь будет исправлена в коде, уже
/// установленные копии сами подхватят исправление при следующем запуске.
/// Компоненты, не найденные ни в одном из списков ниже, остаются со
/// значением по умолчанию из ALTER TABLE (Addon::UserAdded) — то есть тем,
/// что и ожидается для ингредиентов, добавленных вручную через
/// "Редактировать базу" (insert_component отдельно проставляет то же
/// значение явно при вставке новой строки).
///
/// Порядок важен для "Гнилая чешуйка"/"Огненный гриб": эти два имени
/// присутствуют и в RARE_CURIOS_INGREDIENTS, и в SAINTS_INGREDIENTS (см.
/// комментарий в creations.rs), но реальная строка в базе — та, что была
/// вставлена migrate_rare_curios (migrate_creations её не дублирует, видя
/// существующее имя). Поэтому Addon::RareCurios проставляется ПОСЛЕ
/// Addon::SaintsAndSeducers, чтобы для этих двух имён победило правильное
/// значение.
fn migrate_addon_ids(conn: &Connection) -> rusqlite::Result<()> {
    let mut has_addon = false;
    {
        let mut stmt = conn.prepare("PRAGMA table_info(components)")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let name: String = row.get(1)?;
            if name == "addon" {
                has_addon = true;
            }
        }
    }
    if !has_addon {
        conn.execute(
            &format!(
                "ALTER TABLE components ADD COLUMN addon TEXT NOT NULL DEFAULT '{}'",
                Addon::UserAdded.as_str()
            ),
            [],
        )?;
    }

    fn set_addon(conn: &Connection, addon: Addon, names: &[&str]) -> rusqlite::Result<()> {
        for name in names {
            conn.execute(
                "UPDATE components SET addon = ?1 WHERE name = ?2",
                params![addon.as_str(), name],
            )?;
        }
        Ok(())
    }

    // Широкий мазок: весь базовый список — Addon::BaseGame, дальше узкие
    // списки конкретных дополнений переопределяют часть этих строк.
    let base_names: Vec<&str> = INGREDIENTS.iter().map(|(name, _)| *name).collect();
    set_addon(conn, Addon::BaseGame, &base_names)?;
    set_addon(conn, Addon::Dawnguard, DAWNGUARD_INGREDIENTS)?;
    set_addon(conn, Addon::Dragonborn, DRAGONBORN_INGREDIENTS)?;
    set_addon(conn, Addon::Hearthfire, HEARTHFIRE_INGREDIENTS)?;

    set_addon(conn, Addon::Fishing, FISHING_INGREDIENTS)?;
    set_addon(conn, Addon::SaintsAndSeducers, SAINTS_INGREDIENTS)?;
    set_addon(conn, Addon::PlagueOfTheDead, PLAGUE_INGREDIENTS)?;

    // Последним — см. комментарий выше про "Гнилая чешуйка"/"Огненный гриб".
    let rare_curios_names: Vec<&str> = RARE_CURIOS_INGREDIENTS.iter().map(|(name, _)| *name).collect();
    set_addon(conn, Addon::RareCurios, &rare_curios_names)?;

    Ok(())
}

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

/// Список имён компонентов, ограниченный включёнными дополнениями (см.
/// SettingsModal.tsx на фронтенде) — используется выпадающим списком
/// "Компонент" в основном экране поиска. В отличие от load_component_names,
/// который остаётся без фильтра для "Редактировать базу" (там нужен полный
/// список независимо от текущих настроек фильтра).
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

fn addon_placeholders(addons: &[Addon]) -> String {
    addons.iter().map(|_| "?").collect::<Vec<_>>().join(",")
}

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

struct ComponentInfo {
    name: String,
    props: HashSet<String>,
}

/// addons — ингредиенты каких дополнений включать (см. Addon в addons.rs);
/// пустой список означает "ничего не включено" (осознанное состояние,
/// когда пользователь снял в Настройках все галочки), а не "фильтр не
/// задан" — поэтому сразу возвращаем пустой результат, не обращаясь к БД.
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

/// Минимум эффектов готового зелья, при котором тройка ингредиентов
/// считается "максимальным сочетанием" — кнопка "Максимальное количество".
const MAX_COMBO_MIN_EFFECTS: usize = 4;

/// Ищет тройки компонентов, дающие зелье с MAX_COMBO_MIN_EFFECTS (4) и
/// более эффектами — эффект считается проявившимся, если минимум 2 из 3
/// компонентов тройки обладают этим свойством (та же игровая механика, что
/// и в find_combinations/find_pairs). Оформление результата — по образцу
/// find_pairs: строка "A + B + C", затем эффекты с отступом, между записями
/// пустая строка; сортировка — сначала по числу эффектов (по убыванию),
/// затем по именам компонентов. max_results — то же ограничение количества
/// результатов, что и у find_pairs.
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

#[cfg(test)]
mod rare_curios_tests {
    use super::*;

    /// Используем временный ТОЛЬКО файл базы (по абсолютному пути) — саму
    /// рабочую директорию не трогаем. Раньше здесь также проверялся перенос
    /// картинок из папки images/ в BLOB (полная цепочка миграции) — сейчас
    /// эта миграция уже давно проверена и стабильна, а сама папка images/
    /// не поставляется вместе с проектом (не нужна для работы приложения,
    /// см. migrate_images_to_blob), поэтому тесты на неё больше не
    /// полагаются — проверяют только структуру данных после миграции.
    fn temp_db_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("alch_db_test_{label}_{}.db", std::process::id()))
    }

    #[test]
    fn migration_adds_rare_curios() {
        let db_path = temp_db_path("rare_curios");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let names = load_component_names(&conn).unwrap();
        assert!(names.contains(&"Шляпки гифоломы".to_string()), "Rare Curios компонент не добавлен");
        assert_eq!(names.len(), 110 + 52 + 18 - 2, "неверное общее число компонентов после миграции");

        let props = component_properties(&conn, "Шляпки гифоломы").unwrap();
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

        let (_, description) = component_media(&conn, "Шляпки гифоломы");
        assert!(description.contains("Rare Curios"), "описание не упоминает Rare Curios");

        // Повторный вызов ensure_db (как при втором запуске программы) не
        // должен задваивать компоненты.
        drop(conn);
        let conn2 = ensure_db(&db_path).unwrap();
        let names2 = load_component_names(&conn2).unwrap();
        assert_eq!(names2.len(), 110 + 52 + 18 - 2, "повторная миграция задвоила компоненты");

        drop(conn2);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn creations_migration_adds_new_and_skips_duplicates() {
        let db_path = temp_db_path("creations");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();
        let names = load_component_names(&conn).unwrap();

        // 110 базовых + 52 Rare Curios + 18 из творений, минус 2 дубля
        // ("Гнилая чешуйка" и "Огненный гриб" совпадают с Rare Curios).
        assert_eq!(names.len(), 110 + 52 + 18 - 2);

        assert!(names.contains(&"Смертная плоть".to_string()));
        assert!(names.contains(&"Стеклянный окунь".to_string()));

        let props = component_properties(&conn, "Стеклянный окунь").unwrap();
        assert!(props.contains(&"Повышение искусства убеждать".to_string()));

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn set_component_media_stores_and_clears_blob() {
        let db_path = temp_db_path("set_media");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let bytes = vec![1u8, 2, 3, 4, 5];
        set_component_media(&conn, "Шляпки гифоломы", Some(&bytes), "новое описание").unwrap();
        let (image, description) = component_media(&conn, "Шляпки гифоломы");
        assert_eq!(image, Some(bytes));
        assert_eq!(description, "новое описание");

        // None — картинка убирается (например, пользователь очистил поле).
        set_component_media(&conn, "Шляпки гифоломы", None, "новое описание").unwrap();
        let (image2, _) = component_media(&conn, "Шляпки гифоломы");
        assert_eq!(image2, None);

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn migration_classifies_components_by_addon() {
        let db_path = temp_db_path("addon_classify");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        assert_eq!(component_addon(&conn, "Виноград джазби").unwrap(), Some(Addon::BaseGame));
        assert_eq!(component_addon(&conn, "Жёлтый горноцвет").unwrap(), Some(Addon::Dawnguard));
        assert_eq!(component_addon(&conn, "Вредозобник").unwrap(), Some(Addon::Dragonborn));
        assert_eq!(component_addon(&conn, "Лососёвая икра").unwrap(), Some(Addon::Hearthfire));
        assert_eq!(component_addon(&conn, "Шляпки гифоломы").unwrap(), Some(Addon::RareCurios));
        assert_eq!(component_addon(&conn, "Золотая рыбка").unwrap(), Some(Addon::Fishing));
        assert_eq!(component_addon(&conn, "Кричалка").unwrap(), Some(Addon::SaintsAndSeducers));
        assert_eq!(component_addon(&conn, "Смертная плоть").unwrap(), Some(Addon::PlagueOfTheDead));

        // "Гнилая чешуйка" и "Огненный гриб" — общие имена между Rare
        // Curios и "Святыми и соблазнителями"; реальная строка в базе — от
        // Rare Curios (см. комментарий у migrate_addon_ids), поэтому и
        // addon должен остаться Addon::RareCurios, а не переехать на
        // Addon::SaintsAndSeducers.
        assert_eq!(component_addon(&conn, "Гнилая чешуйка").unwrap(), Some(Addon::RareCurios));
        assert_eq!(component_addon(&conn, "Огненный гриб").unwrap(), Some(Addon::RareCurios));

        // Повторная миграция (как при втором запуске) не должна ничего
        // сломать в разметке.
        drop(conn);
        let conn2 = ensure_db(&db_path).unwrap();
        assert_eq!(component_addon(&conn2, "Гнилая чешуйка").unwrap(), Some(Addon::RareCurios));

        drop(conn2);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn insert_component_is_tagged_user_added() {
        let db_path = temp_db_path("addon_user_added");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let props = [
            "Бешенство".to_string(),
            "Замедление".to_string(),
            "Страх".to_string(),
            "Паралич".to_string(),
        ];
        insert_component(&conn, "Тестовый ингредиент", &props).unwrap();
        assert_eq!(component_addon(&conn, "Тестовый ингредиент").unwrap(), Some(Addon::UserAdded));

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn find_combinations_and_pairs_respect_addon_filter() {
        let db_path = temp_db_path("addon_filter_search");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        // Пустой список дополнений — осознанное "ничего не включено", а не
        // "фильтр не задан": и поиск сочетаний, и список компонентов, и
        // парные сочетания должны сразу вернуть пусто, без похода в SQL.
        let empty_names = load_component_names_filtered(&conn, &[]).unwrap();
        assert!(empty_names.is_empty());
        let empty_combos = find_combinations(&conn, &["Бешенство".to_string()], "", &[]).unwrap();
        assert!(empty_combos.is_empty());
        let empty_pairs = find_pairs(&conn, "", &[], usize::MAX).unwrap();
        assert!(empty_pairs.is_empty());

        // Только Rare Curios: "Шляпки гифоломы" — среди её свойств
        // "Бешенство" (Яд), которое есть минимум у двух ингредиентов Rare
        // Curios ("Дреугский воск" тоже им обладает) — комбинации должны
        // находиться, и все имена в них — компоненты Rare Curios.
        let only_rare_curios = [Addon::RareCurios];
        let names = load_component_names_filtered(&conn, &only_rare_curios).unwrap();
        assert_eq!(names.len(), 52, "в Rare Curios должно быть 52 компонента");
        assert!(names.contains(&"Шляпки гифоломы".to_string()));

        let combos = find_combinations(&conn, &["Бешенство".to_string()], "", &only_rare_curios).unwrap();
        assert!(!combos.is_empty(), "должны найтись сочетания внутри Rare Curios");
        for c in &combos {
            for name in &c.components {
                assert_eq!(
                    component_addon(&conn, name).unwrap(),
                    Some(Addon::RareCurios),
                    "в результат просочился компонент не из Rare Curios: {name}"
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
        let lines = find_max_combinations(&conn, "", &all_addons, usize::MAX).unwrap();
        assert!(!lines.is_empty(), "должны найтись тройки с 4+ эффектами");

        // Построчный вывод разбираем на блоки по образцу find_pairs (блоки
        // разделены пустой строкой).
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

        // Отсортировано по убыванию числа эффектов.
        let mut sorted = effect_counts.clone();
        sorted.sort_by(|a, b| b.cmp(a));
        assert_eq!(effect_counts, sorted);

        // Фильтр "Яд" — все эффекты в каждой найденной тройке должны быть ядами.
        let prop_types = load_property_types(&conn).unwrap();
        let poison_lines = find_max_combinations(&conn, "Яд", &all_addons, usize::MAX).unwrap();
        assert!(!poison_lines.is_empty());
        for line in &poison_lines {
            if line.is_empty() || !line.starts_with("    ") {
                continue;
            }
            let prop = line.trim();
            assert_eq!(prop_types.get(prop).map(|s| s.as_str()), Some("Яд"));
        }

        // Пустой список дополнений — осознанно пусто, без похода в SQL.
        let empty = find_max_combinations(&conn, "", &[], usize::MAX).unwrap();
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

        let unlimited_pairs = find_pairs(&conn, "", &all_addons, usize::MAX).unwrap();
        let total_pairs = count_blocks(&unlimited_pairs);
        assert!(total_pairs > 2, "для теста нужно больше пар, чем лимит");
        let limited_pairs = find_pairs(&conn, "", &all_addons, 2).unwrap();
        assert_eq!(count_blocks(&limited_pairs), 2);

        let unlimited_triples = find_max_combinations(&conn, "", &all_addons, usize::MAX).unwrap();
        let total_triples = count_blocks(&unlimited_triples);
        assert!(total_triples > 2, "для теста нужно больше троек, чем лимит");
        let limited_triples = find_max_combinations(&conn, "", &all_addons, 2).unwrap();
        assert_eq!(count_blocks(&limited_triples), 2);

        // Обрезанный результат — это ровно префикс полного (та же
        // сортировка, просто взяли первые max_results записей).
        assert_eq!(limited_pairs, unlimited_pairs[..limited_pairs.len()]);
        assert_eq!(limited_triples, unlimited_triples[..limited_triples.len()]);

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }
}
