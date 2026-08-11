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
