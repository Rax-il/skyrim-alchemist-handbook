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

use commands::AppState;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // Запоминает размер/позицию окна между запусками — прямой аналог
        // window_width/window_height из settings.rs в egui-версии, только
        // теперь этим занимается сам плагин, а не наш код (по умолчанию
        // сохраняет размер, позицию и состояние максимизации).
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            let db_path = paths::database_path();

            // Если рядом с exe ещё нет alchemist.db (в т.ч. в dev-режиме,
            // где cargo кладёт бинарник в target/debug/, а не в папку
            // проекта) — разворачиваем зашитую в бинарник копию со всеми
            // картинками. Если файл уже есть — эта функция ничего не
            // трогает (правки пользователя не теряются).
            if let Err(e) = paths::ensure_seed_db() {
                eprintln!("Не удалось развернуть встроенную базу: {e}");
            }

            let conn = db::ensure_db(&db_path).map_err(|e| {
                // Без консоли (см. windows_subsystem в main.rs) ошибку нужно
                // показать всплывающим системным окном — иначе пользователь
                // просто не поймёт, почему программа не запустилась. Прямой
                // аналог rfd::MessageDialog в исходной main() из egui-версии.
                let handle = app.handle().clone();
                let message = format!(
                    "Не удалось открыть или создать базу данных ({}):\n{}",
                    db_path.display(),
                    e
                );
                handle
                    .dialog()
                    .message(message)
                    .title("Справочник алхимика")
                    .kind(tauri_plugin_dialog::MessageDialogKind::Error)
                    .blocking_show();
                e
            })?;

            app.manage(AppState { conn: Mutex::new(conn) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_properties,
            commands::get_component_names,
            commands::get_component_names_filtered,
            commands::get_component_properties,
            commands::get_component_properties_with_types,
            commands::get_component_media,
            commands::find_combinations,
            commands::find_pairs,
            commands::find_max_combinations,
            commands::component_exists,
            commands::is_user_added_component,
            commands::has_user_added_components,
            commands::insert_component,
            commands::delete_component,
            commands::update_component_properties,
            commands::set_component_media,
            commands::pick_image_file,
            commands::get_layout,
            commands::save_layout,
            commands::save_scale,
            commands::save_language,
            commands::save_addons,
            commands::save_max_combinations,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
