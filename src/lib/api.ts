// api.ts — тонкий типизированный слой над Tauri invoke(). Каждая функция
// здесь соответствует одной #[tauri::command] в src-tauri/src/commands.rs.
// Имена команд и форма аргументов/ответа должны совпадать 1-в-1 с Rust-стороной.

import { invoke } from "@tauri-apps/api/core";
import type { AddonId } from "./addons";

export type FilterKind = "" | "Улучшение" | "Яд";

// Реальное переключение языка — план B3. До тех пор каждый вызов, которому
// нужен lang, использует эту константу — единая точка замены, когда язык
// станет настоящим state вместо константы.
export const CURRENT_LANG = "ru";

export interface PropertyInfo {
  id: number;
  name: string;
}

export interface ComponentNameInfo {
  id: number;
  name: string;
}

export interface CombinationResult {
  components: number[];
  line: string;
}

export interface PropWithType {
  id: number;
  name: string;
  typ: string;
}

export interface ComponentMedia {
  image_data_url: string | null;
  description: string;
}

export interface PickedImage {
  file_name: string;
  base64: string;
}

export interface Layout {
  side_panel_width: number;
  split_ratio: number;
  scale: string;
  enabled_addons: AddonId[];
  max_combinations: number;
}

export const api = {
  getProperties: (lang: string) => invoke<PropertyInfo[]>("get_properties", { lang }),
  getComponentNames: (lang: string) => invoke<ComponentNameInfo[]>("get_component_names", { lang }),
  getComponentNamesFiltered: (addons: AddonId[], lang: string) =>
    invoke<ComponentNameInfo[]>("get_component_names_filtered", { addons, lang }),
  getComponentProperties: (id: number, lang: string) =>
    invoke<string[]>("get_component_properties", { id, lang }),
  getComponentPropertiesWithTypes: (id: number, lang: string) =>
    invoke<PropWithType[]>("get_component_properties_with_types", { id, lang }),
  getComponentMedia: (id: number, lang: string) => invoke<ComponentMedia>("get_component_media", { id, lang }),

  findCombinations: (selected: number[], filter: FilterKind, addons: AddonId[], lang: string) =>
    invoke<CombinationResult[]>("find_combinations", { selected, filter, addons, lang }),
  findPairs: (filter: FilterKind, addons: AddonId[], maxResults: number, lang: string) =>
    invoke<string[]>("find_pairs", { filter, addons, maxResults, lang }),
  findMaxCombinations: (filter: FilterKind, addons: AddonId[], maxResults: number, lang: string) =>
    invoke<string[]>("find_max_combinations", { filter, addons, maxResults, lang }),

  componentExists: (name: string, lang: string) => invoke<boolean>("component_exists", { name, lang }),
  isUserAddedComponent: (id: number) => invoke<boolean>("is_user_added_component", { id }),
  insertComponent: (name: string, lang: string, props: [number, number, number, number]) =>
    invoke<number>("insert_component", { name, lang, props }),
  deleteComponent: (id: number) => invoke<void>("delete_component", { id }),
  updateComponentProperties: (id: number, props: [number, number, number, number]) =>
    invoke<void>("update_component_properties", { id, props }),
  setComponentMedia: (id: number, lang: string, imageBase64: string | null, description: string) =>
    invoke<void>("set_component_media", { id, lang, imageBase64, description }),

  pickImageFile: () => invoke<PickedImage | null>("pick_image_file"),

  getLayout: () => invoke<Layout>("get_layout"),
  saveLayout: (input: { side_panel_width: number; split_ratio: number }) =>
    invoke<void>("save_layout", { input }),
  saveScale: (scale: string) => invoke<void>("save_scale", { scale }),
  saveAddons: (addons: AddonId[]) => invoke<void>("save_addons", { addons }),
  saveMaxCombinations: (maxCombinations: number) =>
    invoke<void>("save_max_combinations", { maxCombinations }),
};

export const TYPE_BENEFIT: FilterKind = "Улучшение";
export const TYPE_POISON: FilterKind = "Яд";
