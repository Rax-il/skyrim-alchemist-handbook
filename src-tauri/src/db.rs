// db.rs — работа со встроенной SQLite-базой: схема, инициализация тестовыми
// данными, CRUD компонентов и поиск сочетаний/пар. Прямой аналог db.go из
// Go-версии, перенесённый на rusqlite.

use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::addons::Addon;
use crate::creations::{
    CREATION_DESCRIPTIONS, CREATION_INGREDIENTS, CREATION_PROPERTIES,
    FISHING_INGREDIENTS, PLAGUE_INGREDIENTS, SAINTS_INGREDIENTS,
};
use crate::rare_curios::{RARE_CURIOS_DESCRIPTION, RARE_CURIOS_INGREDIENTS, RARE_CURIOS_PROPERTIES};
use crate::seed_data::{DAWNGUARD_INGREDIENTS, DRAGONBORN_INGREDIENTS, HEARTHFIRE_INGREDIENTS, INGREDIENTS, MEDIA, PROPERTIES};
use crate::seed_description_translations::{
    DESCRIPTION_TRANSLATIONS, FISHING_DESCRIPTION_TRANSLATIONS, NAME_OVERRIDES,
    PLAGUE_DESCRIPTION_TRANSLATIONS, RARE_CURIOS_DESCRIPTION_TRANSLATIONS,
    SAINTS_DESCRIPTION_TRANSLATIONS,
};
use crate::seed_translations::TRANSLATIONS;

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
    migrate_i18n(&conn)?;
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

    let media_map: HashMap<&str, &str> = MEDIA.iter().map(|(name, desc)| (*name, *desc)).collect();

    for (name, props) in INGREDIENTS {
        let description = media_map.get(name).copied().unwrap_or("");
        conn.execute(
            "INSERT INTO components (name, description) VALUES (?1, ?2)",
            params![name, description],
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

/// Добавляет колонку description, если её ещё нет, и подтягивает данные из
/// MEDIA туда, где сейчас пусто (не затирая ручные правки). Также убирает
/// legacy-колонку image_url, если она осталась в БД от версий, когда
/// картинки читались из файлов на диске (папка images/) — с тех пор все
/// картинки хранятся прямо в BLOB-колонке image (см. ниже), путь к файлу
/// программе больше не нужен, а сама папка images/ давно не поставляется.
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

    if !has_description {
        conn.execute(
            "ALTER TABLE components ADD COLUMN description TEXT NOT NULL DEFAULT ''",
            [],
        )?;
    }
    if !has_image_blob {
        conn.execute("ALTER TABLE components ADD COLUMN image BLOB", [])?;
    }
    if has_image_url {
        conn.execute("ALTER TABLE components DROP COLUMN image_url", [])?;
    }

    for (name, description) in MEDIA {
        conn.execute(
            "UPDATE components
             SET description = CASE WHEN description = '' THEN ?1 ELSE description END
             WHERE name = ?2",
            params![description, name],
        )?;
    }

    Ok(())
}

/// Создаёт таблицы переводов (если их ещё нет) и наполняет их. Всё внутри
/// одной транзакции — раньше каждый INSERT/UPDATE коммитился отдельно, что
/// на ~2000 строк давало заметную паузу при первом запуске (замерено: ~12
/// секунд, ДО появления окна — совершенно незаметно для пользователя,
/// который решит, что программа не запускается). Финальное ревью плана B1
/// нашло это и ещё три связанных бага, все с общей причиной: миграция была
/// "только-запись" (INSERT OR IGNORE) вместо самовосстанавливающейся.
///
/// Порядок внутри транзакции:
/// 1. Чистит переводы для уже удалённых компонентов/свойств — components.id
///    это alias для SQLite rowid без AUTOINCREMENT, значит id может
///    переиспользоваться после удаления, и новый компонент иначе тихо
///    унаследует чужие старые переводы.
/// 2. lang='ru' — upsert из ТЕКУЩЕГО содержимого components/properties, а не
///    INSERT OR IGNORE — иначе правки пользователя (описание/картинка можно
///    менять через "Редактировать базу") после первого запуска навсегда
///    расходятся с тем, что видно в остальных языках.
/// 3. Остальные языки — полностью перестраиваются из TRANSLATIONS при каждом
///    запуске (DELETE + INSERT, не OR IGNORE), потому что это стопроцентно
///    сгенерированные данные без пользовательского ввода (description для
///    не-ru пока всегда '') — иначе обновлённый seed_translations.rs
///    (например, более точный перевод) никогда не долетит до уже
///    установивших программу пользователей. ВАЖНО: если когда-нибудь
///    появится редактирование описаний не на русском (план B4), эта
///    перестройка должна измениться, чтобы не затирать пользовательский ввод.
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

    let tx = conn.unchecked_transaction()?;

    tx.execute(
        "DELETE FROM component_translations WHERE component_id NOT IN (SELECT id FROM components)",
        [],
    )?;
    tx.execute(
        "DELETE FROM property_translations WHERE property_id NOT IN (SELECT id FROM properties)",
        [],
    )?;

    // SQLite grammar note: "SELECT ... FROM tbl ON CONFLICT ..." is
    // ambiguous with a join-constraint (it would parse "tbl ON CONFLICT..."
    // as a join), and SQLite resolves that ambiguity in favor of the join —
    // producing a "near DO: syntax error". The documented workaround is a
    // WHERE clause on the SELECT so "ON CONFLICT" can't be mistaken for
    // part of the FROM clause; see https://www.sqlite.org/lang_upsert.html.
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
    let mut property_ids: HashMap<String, i64> = HashMap::new();
    {
        let mut stmt = tx.prepare("SELECT id, name FROM properties")?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            property_ids.insert(row.get(1)?, row.get(0)?);
        }
    }

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
    // несуществующей строке — тихий no-op, не ошибка.
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

    // Порядок важен: "Гнилая чешуйка"/"Огненный гриб" входят и в
    // RARE_CURIOS_INGREDIENTS, и в SAINTS_INGREDIENTS (см. migrate_addon_ids
    // выше — тот же конфликт уже решён для колонки addon). Rare Curios —
    // корректный источник для этих двух имён, значит применяется ПОСЛЕДНИМ,
    // чтобы победить.
    apply_group_description(&tx, &component_ids, FISHING_INGREDIENTS, FISHING_DESCRIPTION_TRANSLATIONS)?;
    apply_group_description(&tx, &component_ids, SAINTS_INGREDIENTS, SAINTS_DESCRIPTION_TRANSLATIONS)?;
    apply_group_description(&tx, &component_ids, PLAGUE_INGREDIENTS, PLAGUE_DESCRIPTION_TRANSLATIONS)?;
    let rare_curios_names: Vec<&str> = RARE_CURIOS_INGREDIENTS.iter().map(|(name, _)| *name).collect();
    apply_group_description(&tx, &component_ids, &rare_curios_names, RARE_CURIOS_DESCRIPTION_TRANSLATIONS)?;

    tx.commit()?;
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

    for (name, props) in RARE_CURIOS_INGREDIENTS {
        if existing_names.contains(*name) {
            continue;
        }
        conn.execute(
            "INSERT INTO components (name, description) VALUES (?1, ?2)",
            params![name, RARE_CURIOS_DESCRIPTION],
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

    let description_map: HashMap<&str, &str> = CREATION_DESCRIPTIONS.iter().copied().collect();

    for (name, props) in CREATION_INGREDIENTS {
        if existing_names.contains(*name) {
            continue;
        }
        let description = description_map.get(name).copied().unwrap_or("");
        conn.execute(
            "INSERT INTO components (name, description) VALUES (?1, ?2)",
            params![name, description],
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

fn addon_placeholders(addons: &[Addon]) -> String {
    addons.iter().map(|_| "?").collect::<Vec<_>>().join(",")
}

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

/// Сохраняет картинку (байты выбранного пользователем файла — либо None,
/// чтобы убрать картинку) и описание компонента. Пишет и в legacy-колонку
/// components.description (источник бэкфилла migrate_i18n для lang='ru' —
/// см. B1 — но только для официальных компонентов: у Addon::UserAdded
/// migrate_i18n эту колонку с плана B3 игнорирует, см. её комментарий), и
/// напрямую делает upsert в component_translations для переданного lang —
/// иначе правка не была бы видна до перезапуска программы. lang — настоящий
/// параметр (см. Global Constraints плана B2a, никакого хардкода внутри
/// Rust-кода); используется
/// upsert (INSERT ... ON CONFLICT), а не голый UPDATE, потому что строка
/// component_translations для (id, lang) может ещё не существовать —
/// например, если компонент создан на другом языке. Поведение при
/// редактировании компонента, у которого ещё нет строки ни на одном языке,
/// не определено этим планом — это отдельная задача (B4, UX редактирования
/// при нескольких языках).
pub fn set_component_media(
    conn: &Connection,
    id: i64,
    lang: &str,
    image_bytes: Option<&[u8]>,
    description: &str,
) -> rusqlite::Result<()> {
    conn.execute(
        "UPDATE components SET image = ?1, description = ?2 WHERE id = ?3",
        params![image_bytes, description, id],
    )?;
    // Та же грамматическая неоднозначность "FROM tbl ON CONFLICT", что и в
    // migrate_i18n (см. её комментарий выше) — WHERE id = ?1 здесь и
    // разграничивает FROM-клаузу от ON CONFLICT, и одновременно нужен по
    // смыслу (выбрать конкретный компонент).
    conn.execute(
        "INSERT INTO component_translations (component_id, lang, name, description)
         SELECT ?1, ?2, name, ?3 FROM components WHERE id = ?1
         ON CONFLICT(component_id, lang) DO UPDATE SET description = excluded.description",
        params![id, lang, description],
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
/// (для официальных компонентов — источник, из которого migrate_i18n
/// бэкфиллит lang='ru', см. B1; для Addon::UserAdded с плана B3
/// migrate_i18n эту колонку игнорирует — легаси-запись остаётся чисто
/// историческим артефактом), и напрямую в component_translations для lang, на
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
        let inserted = conn.execute(
            "INSERT INTO component_properties (component_id, property_id)
             SELECT ?1, id FROM properties WHERE id = ?2",
            params![component_id, property_id],
        )?;
        if inserted == 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }
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
/// решает фронтенд (Select не даёт отправить форму без выбора). На случай,
/// если сюда всё же попадёт id, которого нет в properties (component_properties
/// без FOREIGN KEY тихо приняла бы его иначе), каждая вставка проверяется —
/// см. Finding 1 финального ревью B2a.
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
        let inserted = conn
            .execute(
                "INSERT INTO component_properties (component_id, property_id)
                 SELECT ?1, id FROM properties WHERE id = ?2",
                params![component_id, property_id],
            )
            .map_err(|e| e.to_string())?;
        if inserted == 0 {
            return Err(format!("свойство с id {property_id} не найдено"));
        }
    }

    Ok(())
}

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

#[cfg(test)]
mod rare_curios_tests {
    use super::*;

    /// Используем временный ТОЛЬКО файл базы (по абсолютному пути) — саму
    /// рабочую директорию не трогаем. Раньше здесь также проверялся перенос
    /// картинок из папки images/ в BLOB (отдельная миграция,
    /// migrate_images_to_blob) — эта миграция и сама колонка image_url,
    /// через которую она работала, с тех пор удалены целиком: все картинки
    /// давно живут прямо в BLOB-колонке image, файлового пути программе
    /// больше не нужно.
    fn temp_db_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("alch_db_test_{label}_{}.db", std::process::id()))
    }

    fn component_id_by_name(conn: &Connection, name: &str) -> i64 {
        conn.query_row("SELECT id FROM components WHERE name = ?1", params![name], |r| r.get(0))
            .unwrap()
    }

    fn property_id_by_name(conn: &Connection, name: &str) -> i64 {
        conn.query_row("SELECT id FROM properties WHERE name = ?1", params![name], |r| r.get(0))
            .unwrap()
    }

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
    fn set_component_media_stores_and_clears_blob() {
        let db_path = temp_db_path("set_media");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();
        let id = component_id_by_name(&conn, "Шляпки гифоломы");

        let bytes = vec![1u8, 2, 3, 4, 5];
        set_component_media(&conn, id, "ru", Some(&bytes), "новое описание").unwrap();
        let (image, description) = component_media(&conn, id, "ru");
        assert_eq!(image, Some(bytes));
        assert_eq!(description, "новое описание");

        set_component_media(&conn, id, "ru", None, "новое описание").unwrap();
        let (image2, _) = component_media(&conn, id, "ru");
        assert_eq!(image2, None);

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

    /// Final review finding #2: set_component_media раньше хардкодила
    /// lang='ru' во втором запросе, а компонент, созданный на другом языке,
    /// не имеет строки component_translations для 'ru' вовсе — правка либо
    /// молча не долетала бы (UPDATE по 0 строк), либо (что ещё хуже)
    /// правка описания компонента, реально существующего на lang='en',
    /// молча создавала бы для него левую 'ru'-строку. Теперь lang —
    /// настоящий параметр, и используется upsert, а не голый UPDATE, —
    /// проверяем оба свойства: (1) правка компонента, у которого ещё нет
    /// строки component_translations для lang='en' (INSERT-ветка upsert-а,
    /// а не только UPDATE, который уже покрыт
    /// migration_ru_upsert_reflects_user_edits), реально создаёт её с
    /// правильным description; (2) при этом строка для lang='ru' того же
    /// компонента остаётся нетронутой.
    #[test]
    fn set_component_media_upserts_under_explicit_lang() {
        let db_path = temp_db_path("set_media_lang");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();
        let id = component_id_by_name(&conn, "Шляпки гифоломы");

        let (_, ru_description_before) = component_media(&conn, id, "ru");

        // Явно убираем en-строку (даже если migrate_i18n/TRANSLATIONS уже
        // её создали), чтобы тест детерминированно проверял именно
        // INSERT-ветку upsert-а, а не полагался на состав seed_translations.rs.
        conn.execute(
            "DELETE FROM component_translations WHERE component_id = ?1 AND lang = 'en'",
            params![id],
        )
        .unwrap();

        set_component_media(&conn, id, "en", None, "edited english description").unwrap();

        let (_, en_description) = component_media(&conn, id, "en");
        assert_eq!(en_description, "edited english description", "upsert должен создать отсутствовавшую строку component_translations для lang='en'");

        let (_, ru_description_after) = component_media(&conn, id, "ru");
        assert_eq!(ru_description_after, ru_description_before, "правка под lang='en' не должна была задеть строку lang='ru'");

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

    /// Final review finding #1: component_properties не имеет FOREIGN KEY,
    /// поэтому без явной проверки несуществующий property_id тихо
    /// "вставлялся" бы и просто выпадал из всех INNER JOIN properties —
    /// компонент выглядел бы так, будто у него меньше свойств, без единой
    /// ошибки. insert_component теперь проверяет каждый id перед вставкой.
    #[test]
    fn insert_component_rejects_unknown_property_id() {
        let db_path = temp_db_path("insert_unknown_prop");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let props = [
            property_id_by_name(&conn, "Бешенство"),
            property_id_by_name(&conn, "Замедление"),
            property_id_by_name(&conn, "Страх"),
            999_999,
        ];
        let result = insert_component(&conn, "Компонент с плохим свойством", "ru", &props);
        assert!(result.is_err(), "insert_component должна вернуть ошибку на несуществующий property_id");

        drop(conn);
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
        // Плагин Plague of the Dead отсутствует в скачанном архиве
        // официальных строк — TRANSLATIONS (пайплайн Bethesda) не даёт ни
        // одного перевода. Но с плана "механизм перевода описаний" пробел
        // закрывает NAME_OVERRIDES (ручной перевод, не из пайплайна) — 1
        // ru-бэкфилл + 8 записей NAME_OVERRIDES = 9, а не "только ru", как
        // было исторически (название теста сохранено ради истории находки,
        // хотя сейчас оно уже не описывает поведение буквально).
        assert_eq!(translation_count, 9);

        drop(conn);
        let _ = std::fs::remove_file(&db_path);
    }

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

        // Не только количество строк должно остаться прежним — содержимое
        // тоже не должно повредиться повторным прогоном миграции.
        let property_id: i64 = conn2
            .query_row("SELECT id FROM properties WHERE name = ?1", params!["Бешенство"], |r| r.get(0))
            .unwrap();
        let en_name: String = conn2
            .query_row(
                "SELECT name FROM property_translations WHERE property_id = ?1 AND lang = 'en'",
                params![property_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(en_name, "Frenzy", "повторная миграция повредила перевод");

        drop(conn2);
        let _ = std::fs::remove_file(&db_path);
    }

    /// Симулирует БД, унаследованную от версии до удаления image_url
    /// (колонка + вся цепочка чтения картинок из файлов), и проверяет, что
    /// migrate_media() безусловно её убирает при следующем запуске.
    #[test]
    fn migrate_media_drops_legacy_image_url_column() {
        let db_path = temp_db_path("drop_image_url");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();
        conn.execute(
            "ALTER TABLE components ADD COLUMN image_url TEXT NOT NULL DEFAULT ''",
            [],
        )
        .unwrap();
        drop(conn);

        let conn2 = ensure_db(&db_path).unwrap();
        let cols: Vec<String> = {
            let mut stmt = conn2.prepare("PRAGMA table_info(components)").unwrap();
            stmt.query_map([], |r| r.get::<_, String>(1))
                .unwrap()
                .collect::<rusqlite::Result<_>>()
                .unwrap()
        };
        assert!(!cols.contains(&"image_url".to_string()), "image_url должна быть удалена: {cols:?}");

        drop(conn2);
        let _ = std::fs::remove_file(&db_path);
    }

    /// Reviewer finding #2: бэкфилл lang='ru' был INSERT OR IGNORE — правка
    /// описания пользователем через "Редактировать базу" (set_component_media)
    /// после первого запуска навсегда расходилась с component_translations.
    /// Теперь это upsert, обновляющийся при каждом ensure_db.
    #[test]
    fn migration_ru_upsert_reflects_user_edits() {
        let db_path = temp_db_path("i18n_ru_upsert");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let component_id = component_id_by_name(&conn, "Белянка");
        let (_, old_description) = component_media(&conn, component_id, "ru");

        set_component_media(&conn, component_id, "ru", None, "новое описание после правки пользователя").unwrap();

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

    /// Reviewer finding #3: не-ru переводы были INSERT OR IGNORE — если
    /// seed_translations.rs перегенерируют с исправлением, уже запустившие
    /// программу пользователи никогда не получат исправление. Теперь
    /// не-ru языки полностью перестраиваются из TRANSLATIONS при каждом
    /// ensure_db, так что "испорченная" (устаревшая) строка перезаписывается.
    #[test]
    fn migration_non_ru_rebuild_overwrites_stale_data() {
        let db_path = temp_db_path("i18n_en_rebuild");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let component_id: i64 = conn
            .query_row("SELECT id FROM components WHERE name = ?1", params!["Стеклянный окунь"], |r| r.get(0))
            .unwrap();

        conn.execute(
            "UPDATE component_translations SET name = 'WRONG' WHERE lang = 'en' AND component_id = ?1",
            params![component_id],
        )
        .unwrap();
        let stale: String = conn
            .query_row(
                "SELECT name FROM component_translations WHERE component_id = ?1 AND lang = 'en'",
                params![component_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(stale, "WRONG");

        drop(conn);
        let conn2 = ensure_db(&db_path).unwrap();

        let fixed: String = conn2
            .query_row(
                "SELECT name FROM component_translations WHERE component_id = ?1 AND lang = 'en'",
                params![component_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(fixed, "Glassfish", "устаревший en-перевод не был перезаписан заново из TRANSLATIONS");

        drop(conn2);
        let _ = std::fs::remove_file(&db_path);
    }

    /// Reviewer finding #4: components.id/properties.id — plain SQLite
    /// rowid без AUTOINCREMENT, значит id переиспользуется после удаления
    /// строки с максимальным id. Без явной чистки новый компонент, которому
    /// достался переиспользованный id, тихо "наследовал" переводы удалённого
    /// компонента. Теперь migrate_i18n сначала удаляет переводы для id, не
    /// существующих в components/properties.
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

    /// Final review finding #1 (see also insert_component_rejects_unknown_property_id):
    /// та же проверка для update_component_properties, у которой отдельный
    /// путь вставки (DELETE + INSERT цикл, а не один insert при создании).
    #[test]
    fn update_component_properties_rejects_unknown_property_id() {
        let db_path = temp_db_path("update_unknown_prop");
        let _ = std::fs::remove_file(&db_path);
        let conn = ensure_db(&db_path).unwrap();

        let props = [
            property_id_by_name(&conn, "Бешенство"),
            property_id_by_name(&conn, "Замедление"),
            property_id_by_name(&conn, "Страх"),
            property_id_by_name(&conn, "Паралич"),
        ];
        let id = insert_component(&conn, "Тестовый ингредиент 3", "ru", &props).unwrap();

        let bad_props = [
            property_id_by_name(&conn, "Водное дыхание"),
            property_id_by_name(&conn, "Невидимость"),
            property_id_by_name(&conn, "Урон здоровью"),
            999_999,
        ];
        let result = update_component_properties(&conn, id, &bad_props);
        assert!(result.is_err(), "update_component_properties должна вернуть ошибку на несуществующий property_id");

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
}
