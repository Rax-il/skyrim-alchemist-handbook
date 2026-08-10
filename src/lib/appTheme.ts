export type AppThemeName = "default" | "skyrim";

export type AppScaleName = "Мелкий" | "Нормальный" | "Крупный";

// Множитель theme.scale в Mantine — масштабирует ВСЕ rem-based размеры
// компонентов и шрифтов (см. main.tsx) пропорционально, единым числом.
export const SCALE_FACTOR_BY_NAME: Record<AppScaleName, number> = {
  Мелкий: 1 / 1.5,
  Нормальный: 1,
  Крупный: 1.5,
};

// Размер окна при масштабе "Нормальный" — совпадает с width/height в
// src-tauri/tauri.conf.json. Смена масштаба ресайзит окно кратно от этой
// базы (см. SettingsModal.tsx).
export const BASE_WINDOW_WIDTH = 1000;
export const BASE_WINDOW_HEIGHT = 800;

// Ниже этой высоты часть компонентов боковой панели перестаёт помещаться —
// при уменьшении масштаба высота окна не должна опускаться ниже этого
// порога, даже если BASE_WINDOW_HEIGHT * factor даёт меньшее значение.
export const MIN_WINDOW_HEIGHT = 650;
