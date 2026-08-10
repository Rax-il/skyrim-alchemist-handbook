// api.ts — тонкий типизированный слой над Tauri invoke(). Каждая функция
// здесь соответствует одной #[tauri::command] в src-tauri/src/commands.rs.
// Имена команд и форма аргументов/ответа должны совпадать 1-в-1 с Rust-стороной.

import { invoke } from "@tauri-apps/api/core";
import type { AddonId } from "./addons";

export type FilterKind = "" | "Улучшение" | "Яд";

export interface CombinationResult {
  components: string[];
  line: string;
}

export interface PropWithType {
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
  getProperties: () => invoke<string[]>("get_properties"),
  getComponentNames: () => invoke<string[]>("get_component_names"),
  getComponentNamesFiltered: (addons: AddonId[]) => invoke<string[]>("get_component_names_filtered", { addons }),
  getComponentProperties: (name: string) => invoke<string[]>("get_component_properties", { name }),
  getComponentPropertiesWithTypes: (name: string) =>
    invoke<PropWithType[]>("get_component_properties_with_types", { name }),
  getComponentMedia: (name: string) => invoke<ComponentMedia>("get_component_media", { name }),

  findCombinations: (selected: string[], filter: FilterKind, addons: AddonId[]) =>
    invoke<CombinationResult[]>("find_combinations", { selected, filter, addons }),
  findPairs: (filter: FilterKind, addons: AddonId[], maxResults: number) =>
    invoke<string[]>("find_pairs", { filter, addons, maxResults }),
  findMaxCombinations: (filter: FilterKind, addons: AddonId[], maxResults: number) =>
    invoke<string[]>("find_max_combinations", { filter, addons, maxResults }),

  componentExists: (name: string) => invoke<boolean>("component_exists", { name }),
  isUserAddedComponent: (name: string) => invoke<boolean>("is_user_added_component", { name }),
  insertComponent: (name: string, props: [string, string, string, string]) =>
    invoke<void>("insert_component", { name, props }),
  renameComponent: (oldName: string, newName: string) =>
    invoke<void>("rename_component", { oldName, newName }),
  deleteComponent: (name: string) => invoke<void>("delete_component", { name }),
  updateComponentProperties: (name: string, props: [string, string, string, string]) =>
    invoke<void>("update_component_properties", { name, props }),
  setComponentMedia: (name: string, imageBase64: string | null, description: string) =>
    invoke<void>("set_component_media", { name, imageBase64, description }),

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
