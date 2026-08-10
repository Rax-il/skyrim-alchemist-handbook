import { StrictMode, useEffect, useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import "@mantine/core/styles.css";
import "./index.css";
import { Button, createTheme, MantineProvider, Select } from "@mantine/core";
import type { MantineColorsTuple } from "@mantine/core";
import App from "./App.tsx";
import { api } from "./lib/api";
import type { AppScaleName, AppThemeName } from "./lib/appTheme";
import { SCALE_FACTOR_BY_NAME } from "./lib/appTheme";

// Официальный способ (Styles API + theme.components) переопределить подсветку
// пункта выпадающего списка для ВСЕХ Select в приложении разом — дефолтная
// подсветка Mantine при наведении/клавиатурной навигации (атрибут
// data-combobox-active) слишком бледная, из-за чего непонятно, какой пункт
// сейчас активен. Глобальный CSS-селектор снаружи (index.css) себя не
// оправдал — Styles API гарантированно выигрывает по специфичности, так как
// Mantine сам генерирует под него правило в своём стилевом слое.
const components = {
  Select: Select.extend({
    styles: {
      option: {
        "&[data-combobox-active]": {
          backgroundColor: "var(--mantine-color-blue-2)",
          color: "var(--mantine-color-blue-9)",
        },
      },
    },
  }),
};

const baseTheme = createTheme({ components });

// Тема "Skyrim". Один и тот же массив используется и как colors.dark
// (фон/рамки для встроенных компонентов Mantine), и как colors.gray (наши
// собственные инлайн-рамки в App.tsx/SettingsModal.tsx завязаны на
// var(--mantine-color-gray-3)) — так рамки и там, и там получают один и тот
// же золотой тон. Индексы 0-2 — текст, 3-5 — рамки/разделители (золото),
// 6-9 — фон (инпуты/панели/модалки) — строго нейтральный чёрный, без
// оттенка золота.
const skyrimScale: MantineColorsTuple = [
  "#EAD9A0", // 0 — основной текст
  "#D8C48A", // 1
  "#B9A26A", // 2 — приглушённый/dimmed текст
  "#8F7A4E", // 3 — акцент (чекбоксы/radio), приглушённое золото
  "#6B5B3E", // 4 — рамки/разделители
  "#4A3F2C", // 5 — рамки/разделители (темнее)
  "#161616", // 6 — фон инпутов/Select, нейтральный чёрный
  "#121212", // 7 — фон панелей/модалок, нейтральный чёрный
  "#0D0D0D", // 8
  "#050505", // 9 — самый тёмный фон
];

const skyrimComponents = {
  ...components,
  // Заливка кнопок (filled/light/default — любой variant) убрана в пользу
  // плоского чёрного фона с золотыми рамкой и текстом, по образцу игровых
  // меню, где кнопок с цветной заливкой нет вообще.
  Button: Button.extend({
    vars: () => ({
      root: {
        "--button-bg": "var(--mantine-color-dark-7)",
        "--button-hover": "var(--mantine-color-dark-6)",
        "--button-color": "var(--mantine-color-dark-0)",
        "--button-bd": "1px solid var(--mantine-color-dark-4)",
      },
    }),
  }),
};

const skyrimTheme = createTheme({
  primaryColor: "dark",
  primaryShade: 3,
  colors: {
    dark: skyrimScale,
    gray: skyrimScale,
  },
  components: skyrimComponents,
});

function Root() {
  const [appTheme, setAppTheme] = useState<AppThemeName>("default");
  const [scale, setScale] = useState<AppScaleName>("Нормальный");
  const isSkyrim = appTheme === "skyrim";

  // Подхватить сохранённый масштаб при старте — сам масштаб не трогает
  // размер окна (это делает только явное "Применить" в SettingsModal), окно
  // при запуске по-прежнему восстанавливает tauri-plugin-window-state.
  useEffect(() => {
    api.getLayout().then((l) => {
      if (l.scale in SCALE_FACTOR_BY_NAME) setScale(l.scale as AppScaleName);
    });
  }, []);

  // theme.scale — официальный механизм Mantine: множитель, на который
  // домножаются ВСЕ rem-based размеры (шрифты, отступы, высоты полей и
  // т.д.) через CSS-переменную --mantine-scale. Совмещаем его с уже
  // выбранным оформлением (base/skyrim), а не создаём ещё одну пару тем.
  const activeTheme = useMemo(
    () =>
      createTheme({
        ...(isSkyrim ? skyrimTheme : baseTheme),
        scale: SCALE_FACTOR_BY_NAME[scale],
      }),
    [isSkyrim, scale],
  );

  return (
    <MantineProvider
      theme={activeTheme}
      defaultColorScheme="auto"
      forceColorScheme={isSkyrim ? "dark" : undefined}
    >
      <App
        appTheme={appTheme}
        onAppThemeChange={setAppTheme}
        scale={scale}
        onScaleChange={setScale}
      />
    </MantineProvider>
  );
}

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <Root />
  </StrictMode>,
);
