// commands.rs — набор #[tauri::command], которые вызывает фронтенд через
// invoke(). Вся бизнес-логика (SQL, поиск сочетаний, миграции) остаётся в
// db.rs один в один как в egui-версии — здесь только (де)сериализация в
// JSON и работа с общим соединением через Mutex.

use base64::{engine::general_purpose::STANDARD, Engine as _};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::State;

use crate::addons::Addon;
use crate::db;
use crate::layout::{self, Layout};

pub struct AppState {
    pub conn: Mutex<Connection>,
}

fn map_err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

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

#[derive(Serialize)]
pub struct ComponentMedia {
    /// Картинка в виде data-URL (data:image/*;base64,...), готовая для
    /// прямой подстановки в <img src=...> на фронтенде — без отдельного
    /// декодирования на JS-стороне.
    pub image_data_url: Option<String>,
    pub description: String,
}

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
pub fn has_user_added_components(state: State<AppState>) -> Result<bool, String> {
    let conn = state.conn.lock().map_err(map_err)?;
    db::has_user_added_components(&conn).map_err(map_err)
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
    lang: String,
    image_base64: Option<String>,
    description: String,
) -> Result<(), String> {
    let conn = state.conn.lock().map_err(map_err)?;
    let bytes = image_base64.map(|s| STANDARD.decode(s)).transpose().map_err(map_err)?;
    db::set_component_media(&conn, id, &lang, bytes.as_deref(), &description).map_err(map_err)
}

#[derive(Serialize)]
pub struct PickedImage {
    pub file_name: String,
    /// «Сырой» base64 без префикса — фронтенд сам собирает data-URL для
    /// превью и передаёт то же значение обратно в set_component_media.
    pub base64: String,
}

/// Открывает системный диалог выбора файла (аналог rfd::FileDialog в
/// egui-версии) и читает выбранный файл. Возвращает None, если пользователь
/// отменил диалог.
#[tauri::command]
pub async fn pick_image_file(app: tauri::AppHandle) -> Result<Option<PickedImage>, String> {
    use tauri_plugin_dialog::DialogExt;

    let file_path = app
        .dialog()
        .file()
        .add_filter("Изображения", &["png", "jpg", "jpeg", "gif", "webp", "bmp"])
        .set_title("Выберите изображение компонента")
        .blocking_pick_file();

    let Some(file_path) = file_path else {
        return Ok(None);
    };

    let path = file_path.into_path().map_err(map_err)?;
    let bytes = std::fs::read(&path).map_err(map_err)?;
    let file_name = path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string_lossy().to_string());

    Ok(Some(PickedImage {
        file_name,
        base64: STANDARD.encode(bytes),
    }))
}

#[tauri::command]
pub fn get_layout() -> Layout {
    layout::load()
}

#[derive(Deserialize)]
pub struct LayoutInput {
    pub side_panel_width: f64,
    pub split_ratio: f64,
}

#[tauri::command]
pub fn save_layout(input: LayoutInput) {
    layout::save_panel(input.side_panel_width, input.split_ratio);
}

#[tauri::command]
pub fn save_scale(scale: String) {
    layout::save_scale(&scale);
}

#[tauri::command]
pub fn save_language(language: String) {
    layout::save_language(&language);
}

#[tauri::command]
pub fn save_addons(addons: Vec<String>) {
    layout::save_addons(&addons);
}

#[tauri::command]
pub fn save_max_combinations(max_combinations: u32) {
    layout::save_max_combinations(max_combinations);
}

/// Определяет MIME-тип по сигнатуре первых байтов (PNG/JPEG/GIF/WebP/BMP —
/// тот же набор форматов, что принимал диалог выбора файла в editor.rs
/// исходной версии). image/png как запасной вариант, если сигнатура не
/// распознана — редкий случай, картинка всё равно почти наверняка одна из
/// этих пяти.
fn sniff_mime(bytes: &[u8]) -> &'static str {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if bytes.starts_with(b"\xFF\xD8\xFF") {
        "image/jpeg"
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        "image/gif"
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else if bytes.starts_with(b"BM") {
        "image/bmp"
    } else {
        "image/png"
    }
}

fn to_data_url(bytes: &[u8]) -> String {
    format!("data:{};base64,{}", sniff_mime(bytes), STANDARD.encode(bytes))
}
