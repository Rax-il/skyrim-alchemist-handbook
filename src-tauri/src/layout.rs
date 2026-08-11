// layout.rs — сохранение/восстановление раскладки внутри окна (ширина
// левой панели с фильтрами, позиция разделителя между верхней и нижней
// панелью) и масштаба интерфейса. Аналог settings.rs из egui-версии, но без
// window_width/height — это теперь берёт на себя tauri-plugin-window-state
// (см. lib.rs), кроме случаев смены масштаба — тогда окно ресайзится явно
// с фронтенда (см. SettingsModal.tsx).

use serde::{Deserialize, Serialize};

use crate::addons::Addon;

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

/// "Все дополнения включены" — состояние по умолчанию для только что
/// установленного приложения и запасной вариант при повреждённом/старом
/// settings.json без этого поля.
fn default_addons() -> Vec<String> {
    Addon::ALL.iter().map(|a| a.as_str().to_string()).collect()
}

fn default_max_combinations() -> u32 {
    100
}

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

/// Сохраняет набор включённых дополнений, не трогая остальную раскладку —
/// по той же причине, что и save_panel/save_scale (см. их комментарии).
pub fn save_addons(addons: &[String]) {
    let mut current = load();
    current.enabled_addons = addons.to_vec();
    save(&current);
}

/// Сохраняет лимит числа результатов "Парных"/"Тройных сочетаний", не
/// трогая остальную раскладку — по той же причине, что и save_addons.
pub fn save_max_combinations(max_combinations: u32) {
    let mut current = load();
    current.max_combinations = max_combinations;
    save(&current);
}

fn save(layout: &Layout) {
    let path = crate::paths::settings_path();
    if let Ok(data) = serde_json::to_string(layout) {
        let _ = std::fs::write(path, data);
    }
}

/// Сохраняет ширину боковой панели и позицию разделителя. Читает текущий
/// файл настроек и обновляет только эти два поля, чтобы не затереть scale,
/// сохранённый независимо через save_scale (иначе более старое состояние
/// одной части настроек на фронтенде могло бы откатить другую).
pub fn save_panel(side_panel_width: f64, split_ratio: f64) {
    let mut current = load();
    current.side_panel_width = side_panel_width;
    current.split_ratio = split_ratio;
    save(&current);
}

/// Сохраняет масштаб, не трогая раскладку панелей — по той же причине.
pub fn save_scale(scale: &str) {
    let mut current = load();
    current.scale = scale.to_string();
    save(&current);
}

/// Сохраняет выбранный язык, не трогая остальную раскладку — по той же
/// причине, что и save_scale/save_addons.
pub fn save_language(language: &str) {
    let mut current = load();
    current.language = language.to_string();
    save(&current);
}
