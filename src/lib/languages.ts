// languages.ts — официальные локализации Skyrim Special Edition (+
// китайский, только текстовый перевод, без озвучки) — общий словарь для
// SettingsModal (выбор языка) и EditorModal (подпись в диалоге "Новый
// ингредиент"). value должен совпадать с кодами, которые понимает бэкенд
// (component_translations.lang и т.д.) — zh-Hant, не zh (см.
// docs/superpowers/specs/2026-08-10-i18n-storage-design.md).
//
// Флаги — SVG-файлы (flag-icons, MIT), а не emoji: Windows не рендерит
// составные flag-emoji символы как картинку (показывает буквенный код
// страны), Linux/Mac — рендерят. SVG выглядит одинаково на всех ОС.

import flagRu from "../assets/flags/ru.svg";
import flagGb from "../assets/flags/en.svg";
import flagFr from "../assets/flags/fr.svg";
import flagDe from "../assets/flags/de.svg";
import flagIt from "../assets/flags/it.svg";
import flagEs from "../assets/flags/es.svg";
import flagPl from "../assets/flags/pl.svg";
import flagJa from "../assets/flags/ja.svg";
import flagZhHant from "../assets/flags/zh-hant.svg";

export interface LanguageOption {
  value: string;
  label: string;
  flag: string;
}

export const LANGUAGE_OPTIONS: LanguageOption[] = [
  { value: "ru", label: "Русский", flag: flagRu },
  { value: "en", label: "English", flag: flagGb },
  { value: "fr", label: "Français", flag: flagFr },
  { value: "de", label: "Deutsch", flag: flagDe },
  { value: "it", label: "Italiano", flag: flagIt },
  { value: "es", label: "Español", flag: flagEs },
  { value: "pl", label: "Polski", flag: flagPl },
  { value: "ja", label: "日本語", flag: flagJa },
  { value: "zh-Hant", label: "中文", flag: flagZhHant },
];
