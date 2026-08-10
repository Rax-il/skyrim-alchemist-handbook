// addons.ts — идентификаторы источников ингредиентов, зеркало Addon из
// src-tauri/src/addons.rs. Строковые значения должны 1-в-1 совпадать с
// Addon::as_str() на Rust-стороне (это snake_case slug, а не отображаемое
// название — само название вынесено отдельно в ADDON_LABELS, чтобы смена
// языка интерфейса в будущем его не задевала).

export type AddonId =
  | "base_game"
  | "dawnguard"
  | "dragonborn"
  | "hearthfire"
  | "rare_curios"
  | "fishing"
  | "saints_and_seducers"
  | "plague_of_the_dead"
  | "user_added";

// Порядок — как отображается в SettingsModal ("Базовый набор" всегда
// первым в списке).
export const ADDON_CHECKBOX_IDS: AddonId[] = [
  "base_game",
  "dawnguard",
  "dragonborn",
  "hearthfire",
  "rare_curios",
  "fishing",
  "saints_and_seducers",
  "plague_of_the_dead",
  "user_added",
];

export const ALL_ADDON_IDS: AddonId[] = ADDON_CHECKBOX_IDS;

export const ADDON_LABELS: Record<AddonId, string> = {
  base_game: "Базовый набор",
  dawnguard: "Dawnguard",
  dragonborn: "Dragonborn",
  hearthfire: "Hearthfire",
  rare_curios: "Rare Curios",
  fishing: "Рыбалка",
  saints_and_seducers: "Святые и соблазнители",
  plague_of_the_dead: "Чума мертвецов",
  user_added: "Добавлено пользователем",
};
