# Перевод интерфейса приложения Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** перевести весь текст интерфейса приложения (меню, кнопки, лейблы,
тултипы, диалоги, заголовок окна) на все 8 не-ru языков через
`react-i18next`, синхронизировано с уже существующим `language` state в
`App.tsx`.

**Architecture:** Task 1 — установка зависимостей и минимальный мост
(`src/i18n/index.ts`, синхронизация `language` → `i18n.changeLanguage` +
заголовок окна). Task 2 — общий namespace `common.*` для строк, повторяющихся
в ≥2 компонентах (заголовки модалок, кнопки Отмена/Продолжить, плейсхолдер
"не выбрано" и т.п.) — раньше эти строки, включая `EMPTY_OPTION`, были
независимо продублированы как локальные константы в разных файлах. Tasks
3-9 — по одной задаче на компонент: извлечение захардкоженных строк в ключи
`t()` (реальный русский текст, взят из уже существующего кода — механическое
действие) + перевод этих ключей сразу на все 8 языков (весь объём делаем
одним заходом, без разделения на "код" и "контент" по разным сессиям — в
отличие от перевода описаний ингредиентов, где это было оправдано их
объёмом, здесь ~90 строк допускают перевод сразу по ходу извлечения).

**Tech Stack:** `react-i18next` + `i18next` (новая зависимость, нет
аналогов в проекте — обоснование выбора над самописным `t()` в дизайн-доке).
TypeScript/React/Mantine — существующий стек. Тестового раннера на фронтенде
в проекте нет — верификация как во всех предыдущих фронтенд-планах (B2b,
B3+B4, hint-icons): `npx tsc -b` + `npm run build` + `npm run lint`.

## Global Constraints

- **Дизайн-документ:** `docs/superpowers/specs/2026-08-12-ui-interface-translation-design.md`
  — читать целиком перед началом, здесь только то, что нужно для реализации.
- **9 языков, коды как в `LANGUAGE_OPTIONS`** (`src/lib/languages.ts`): `ru`,
  `en`, `fr`, `de`, `it`, `es`, `pl`, `ja`, `zh-Hant`. Ресурсы —
  `src/i18n/locales/<lang>/translation.json`, один namespace `translation`,
  статический импорт (объём не требует ленивой загрузки).
- **`fallbackLng: "ru"`** — если для ключа забыт перевод на каком-то языке,
  показывается русский текст, а не пустая строка/ключ.
- **`App.tsx`'s `language` state — единственный источник истины**, ничего не
  заводить параллельно. Синхронизация — `i18n.changeLanguage(language)` в
  `useEffect`, тем же паттерном, что уже используют существующие эффекты
  (`api.getProperties(language)` и т.п.).
- **КРИТИЧНО — три места хранят фиксированный русский токен как ВНУТРЕННЕЕ
  ЗНАЧЕНИЕ (не только отображаемый лейбл), НЕ ТРОГАТЬ эти значения:**
  1. `AppScaleName` (`"Мелкий" | "Нормальный" | "Крупный"`, `src/lib/appTheme.ts`)
     — персистится дословно в `alchemist_settings.json` И валидируется на
     Rust-стороне (`src-tauri/src/layout.rs:12`, свой `SCALE_OPTIONS: &[&str]`).
  2. `FilterKind` (`"" | "Улучшение" | "Яд"`, `src/lib/api.ts`) — сравнивается
     на Rust-стороне с `TYPE_BENEFIT`/`TYPE_POISON` (`src-tauri/src/db.rs:24-25`,
     оба буквально `"Улучшение"`/`"Яд"`).
  3. Локальный `theme` state в `SettingsModal.tsx` (строки `"Системная"`/
     `"Светлая"`/`"Тёмная"`/`"Skyrim"`, через `THEME_LABEL_BY_SCHEME`/
     `THEME_SCHEME_BY_LABEL`) — не персистится как текст на Rust-сторону
     (`appTheme` prop — только `"default"`/`"skyrim"`), но той же логики ради
     тоже не трогаем сам токен, переводим только то, что видит пользователь.

  Во всех трёх случаях — **значение (`value` у `Select`/`Radio`, ключи
  мапов) остаётся русским как есть, переводится только `label`**, тем же
  приёмом, что уже применён в `ADDON_LABELS` (`src/lib/addons.ts`) —
  см. комментарий в начале этого файла, он написан заранее именно под эту
  будущую задачу. Перепутать value и label здесь означает тихо сломать
  фильтрацию бэкенда или персистентность настроек.
- **`ADDON_LABELS` (`src/lib/addons.ts`) не редактируется вообще** — файл
  используется в единственном месте (`SettingsModal.tsx:255`), перевод
  добавляется на уровне вызова, не в этом файле (см. Task 5). `dawnguard`/
  `dragonborn`/`hearthfire`/`rare_curios` — фирменные названия DLC, не
  переводятся ни на одном языке (уже так в текущей русской версии).
- **Заголовок окна** — это Tauri window title, не React DOM. Устанавливается
  через `getCurrentWindow().setTitle(...)` из `@tauri-apps/api/window`
  (уже импортируется в `App.tsx`), не через JSX/`t()` напрямую.
  `src-tauri/tauri.conf.json`'s `app.windows[0].title` не редактируется —
  это только начальное значение при самом первом кадре до отработки JS,
  реальный заголовок дальше всегда управляется рантаймом.
- **Интерполяция** — `{{paramName}}`, синтаксис i18next по умолчанию
  (`interpolation: { escapeValue: false }`, не HTML-контекст, экранирование
  не нужно).
- **Плюрализация** — суффиксы `_one`/`_few`/`_many`/`_other` в JSON (только
  у ключей, где это указано ниже). `ru`/`pl` — все 4 формы. `en`/`fr`/`de`/
  `it`/`es` — только `_one`/`_other`. `ja`/`zh-Hant` — только `_other` (в
  CLDR у этих языков единственная форма).
- **Нет фронтенд-тестового раннера** — верификация каждой задачи:
  `npx tsc -b && npm run build && npm run lint`, все должны быть чистыми.
  Визуальная проверка — скриншот от пользователя (не поднимать dev-сервер
  самостоятельно), по одному на завершение Task 1 (заголовок окна) и в
  конце всего плана (несколько языков).

---

### Task 1: Зависимости + мост `i18next` + заголовок окна

**Files:**
- Modify: `package.json`, `package-lock.json` (новые зависимости)
- Modify: `tsconfig.app.json` (добавить `resolveJsonModule: true` —
  требуется для `import ru from "./locales/ru/translation.json"`)
- Create: `src/i18n/index.ts`
- Create: `src/i18n/locales/ru/translation.json`
- Create: `src/i18n/locales/en/translation.json`
- Create: `src/i18n/locales/fr/translation.json`
- Create: `src/i18n/locales/de/translation.json`
- Create: `src/i18n/locales/it/translation.json`
- Create: `src/i18n/locales/es/translation.json`
- Create: `src/i18n/locales/pl/translation.json`
- Create: `src/i18n/locales/ja/translation.json`
- Create: `src/i18n/locales/zh-Hant/translation.json`
- Modify: `src/App.tsx:1-4` (импорты), новый `useEffect` рядом с остальными
  эффектами (после блока сохранения раскладки, до блока с `getProperties`)

**Interfaces:**
- Produces (используется всеми следующими задачами):
  - `export default i18n` из `src/i18n/index.ts` — глобальный экземпляр
    `i18next`, уже инициализированный (`.init()` — часть импорта, побочный
    эффект модуля).
  - Ключ `appTitle` в каждом `translation.json` — единственный ключ,
    добавляемый этой задачей помимо служебного каркаса.

- [ ] **Step 1: Установить зависимости**

```bash
npm install react-i18next i18next
```

- [ ] **Step 2: Добавить `resolveJsonModule` в `tsconfig.app.json`**

В `compilerOptions`, рядом с `"allowArbitraryExtensions": true`:

```json
    "resolveJsonModule": true,
```

- [ ] **Step 3: Создать 9 файлов локалей — только ключ `appTitle`**

`src/i18n/locales/ru/translation.json`:
```json
{
  "appTitle": "Справочник алхимика"
}
```

`src/i18n/locales/en/translation.json`:
```json
{
  "appTitle": "Alchemist's Handbook"
}
```

`src/i18n/locales/fr/translation.json`:
```json
{
  "appTitle": "Manuel de l'alchimiste"
}
```

`src/i18n/locales/de/translation.json`:
```json
{
  "appTitle": "Alchemisten-Handbuch"
}
```

`src/i18n/locales/it/translation.json`:
```json
{
  "appTitle": "Manuale dell'alchimista"
}
```

`src/i18n/locales/es/translation.json`:
```json
{
  "appTitle": "Manual del alquimista"
}
```

`src/i18n/locales/pl/translation.json`:
```json
{
  "appTitle": "Podręcznik alchemika"
}
```

`src/i18n/locales/ja/translation.json`:
```json
{
  "appTitle": "錬金術師の手引き"
}
```

`src/i18n/locales/zh-Hant/translation.json`:
```json
{
  "appTitle": "煉金術士手冊"
}
```

- [ ] **Step 4: Создать `src/i18n/index.ts`**

```ts
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import ru from "./locales/ru/translation.json";
import en from "./locales/en/translation.json";
import fr from "./locales/fr/translation.json";
import de from "./locales/de/translation.json";
import it from "./locales/it/translation.json";
import es from "./locales/es/translation.json";
import pl from "./locales/pl/translation.json";
import ja from "./locales/ja/translation.json";
import zhHant from "./locales/zh-Hant/translation.json";

i18n.use(initReactI18next).init({
  resources: {
    ru: { translation: ru },
    en: { translation: en },
    fr: { translation: fr },
    de: { translation: de },
    it: { translation: it },
    es: { translation: es },
    pl: { translation: pl },
    ja: { translation: ja },
    "zh-Hant": { translation: zhHant },
  },
  lng: "ru",
  fallbackLng: "ru",
  interpolation: { escapeValue: false },
});

export default i18n;
```

- [ ] **Step 5: Подключить в `App.tsx` — синхронизация языка + заголовок окна**

В блоке импортов (после `import { getCurrentWindow } from "@tauri-apps/api/window";`):

```tsx
import i18n from "./i18n";
```

Новый эффект — сразу после эффекта сохранения раскладки (после строки с
`}, [sidePanelWidth, splitRatio]);`) и до эффекта загрузки списка
свойств/компонентов:

```tsx
  // --- Синхронизация языка интерфейса с i18next + заголовок окна (Tauri
  // window title, не React DOM — не переводится через JSX). ---
  useEffect(() => {
    i18n.changeLanguage(language).then(() => {
      getCurrentWindow().setTitle(i18n.t("appTitle"));
    });
  }, [language]);
```

- [ ] **Step 6: Проверить сборку**

```bash
npx tsc -b && npm run build && npm run lint
```
Ожидается: всё зелёное, без ошибок.

- [ ] **Step 7: Попросить у пользователя скриншот**

Запустить `npm run tauri dev` (или попросить пользователя запустить) и
проверить, что заголовок окна ОС по-прежнему "Справочник алхимика" (язык
приложения ещё не переключался, `DEFAULT_LANGUAGE = "ru"`) — регрессии
названия окна быть не должно на этом шаге, само по себе видимых изменений
в интерфейсе ещё нет. Экран не поднимать самостоятельно — попросить
пользователя подтвердить визуально.

- [ ] **Step 8: Commit**

```bash
git add package.json package-lock.json tsconfig.app.json src/i18n src/App.tsx
git commit -m "feat: bootstrap react-i18next, sync language state, translate window title"
```

---

### Task 2: `common.*` — общие ключи, используемые в ≥2 компонентах

**Files:**
- Modify: все 9 `src/i18n/locales/<lang>/translation.json`

**Interfaces:**
- Consumes: файлы из Task 1 (уже содержат `appTitle`, новые ключи
  добавляются рядом).
- Produces (используется Tasks 3-9): ключи `common.error`, `common.attention`,
  `common.done`, `common.confirmTitle`, `common.cancel`, `common.continue`,
  `common.close`, `common.notSelected`, `common.noImage`,
  `common.propertyLabel` (интерполяция `{{index}}`).

- [ ] **Step 1: Добавить `common` в каждый из 9 файлов**

`src/i18n/locales/ru/translation.json` (полностью, добавляем секцию `common`
рядом с уже существующим `appTitle`):
```json
{
  "appTitle": "Справочник алхимика",
  "common": {
    "error": "Ошибка",
    "attention": "Внимание",
    "done": "Готово",
    "confirmTitle": "Подтверждение",
    "cancel": "Отмена",
    "continue": "Продолжить",
    "close": "Закрыть",
    "notSelected": "— не выбрано —",
    "noImage": "Нет изображения",
    "propertyLabel": "Свойство {{index}}"
  }
}
```

`en`:
```json
  "common": {
    "error": "Error",
    "attention": "Attention",
    "done": "Done",
    "confirmTitle": "Confirmation",
    "cancel": "Cancel",
    "continue": "Continue",
    "close": "Close",
    "notSelected": "— not selected —",
    "noImage": "No image",
    "propertyLabel": "Property {{index}}"
  }
```

`fr`:
```json
  "common": {
    "error": "Erreur",
    "attention": "Attention",
    "done": "Terminé",
    "confirmTitle": "Confirmation",
    "cancel": "Annuler",
    "continue": "Continuer",
    "close": "Fermer",
    "notSelected": "— non sélectionné —",
    "noImage": "Aucune image",
    "propertyLabel": "Propriété {{index}}"
  }
```

`de`:
```json
  "common": {
    "error": "Fehler",
    "attention": "Achtung",
    "done": "Fertig",
    "confirmTitle": "Bestätigung",
    "cancel": "Abbrechen",
    "continue": "Fortfahren",
    "close": "Schließen",
    "notSelected": "— nicht ausgewählt —",
    "noImage": "Kein Bild",
    "propertyLabel": "Eigenschaft {{index}}"
  }
```

`it`:
```json
  "common": {
    "error": "Errore",
    "attention": "Attenzione",
    "done": "Fatto",
    "confirmTitle": "Conferma",
    "cancel": "Annulla",
    "continue": "Continua",
    "close": "Chiudi",
    "notSelected": "— non selezionato —",
    "noImage": "Nessuna immagine",
    "propertyLabel": "Proprietà {{index}}"
  }
```

`es`:
```json
  "common": {
    "error": "Error",
    "attention": "Atención",
    "done": "Hecho",
    "confirmTitle": "Confirmación",
    "cancel": "Cancelar",
    "continue": "Continuar",
    "close": "Cerrar",
    "notSelected": "— no seleccionado —",
    "noImage": "Sin imagen",
    "propertyLabel": "Propiedad {{index}}"
  }
```

`pl`:
```json
  "common": {
    "error": "Błąd",
    "attention": "Uwaga",
    "done": "Gotowe",
    "confirmTitle": "Potwierdzenie",
    "cancel": "Anuluj",
    "continue": "Kontynuuj",
    "close": "Zamknij",
    "notSelected": "— nie wybrano —",
    "noImage": "Brak obrazu",
    "propertyLabel": "Właściwość {{index}}"
  }
```

`ja`:
```json
  "common": {
    "error": "エラー",
    "attention": "注意",
    "done": "完了",
    "confirmTitle": "確認",
    "cancel": "キャンセル",
    "continue": "続行",
    "close": "閉じる",
    "notSelected": "— 未選択 —",
    "noImage": "画像なし",
    "propertyLabel": "特性{{index}}"
  }
```

`zh-Hant`:
```json
  "common": {
    "error": "錯誤",
    "attention": "注意",
    "done": "完成",
    "confirmTitle": "確認",
    "cancel": "取消",
    "continue": "繼續",
    "close": "關閉",
    "notSelected": "— 未選擇 —",
    "noImage": "無圖片",
    "propertyLabel": "屬性{{index}}"
  }
```

- [ ] **Step 2: Проверить, что JSON валиден и сборка проходит**

```bash
npx tsc -b && npm run build && npm run lint
```
Ожидается: всё зелёное (ключи пока никем не используются, но `resolveJsonModule`
+ статический импорт в `src/i18n/index.ts` уже разбирает файлы — синтаксическая
ошибка в любом JSON провалит `tsc -b`/`vite build`).

- [ ] **Step 3: Commit**

```bash
git add src/i18n
git commit -m "feat: add common i18n namespace shared across components"
```

---

### Task 3: `App.tsx` — меню и оставшиеся строки

**Files:**
- Modify: `src/App.tsx`
- Modify: все 9 `src/i18n/locales/<lang>/translation.json`

**Interfaces:**
- Consumes: `common.*` (Task 2), `i18n` default export (Task 1).
- Produces: ничем не потребляется другими задачами (App.tsx — верхний
  уровень).

- [ ] **Step 1: Добавить ключи `menu.*` и `app.*` во все 9 файлов**

`ru` (добавить рядом с `common`):
```json
  "menu": {
    "file": "Файл",
    "editDatabase": "Редактировать базу",
    "settings": "Настройки",
    "exit": "Выход",
    "about": "О программе"
  },
  "app": {
    "resultsFound_one": "Найдено {{count}} комбинация",
    "resultsFound_few": "Найдено {{count}} комбинации",
    "resultsFound_many": "Найдено {{count}} комбинаций",
    "resultsFound_other": "Найдено {{count}} комбинации",
    "resultsDisplayed_one": "Отображено {{count}} комбинация",
    "resultsDisplayed_few": "Отображено {{count}} комбинации",
    "resultsDisplayed_many": "Отображено {{count}} комбинаций",
    "resultsDisplayed_other": "Отображено {{count}} комбинации",
    "slowOperationConfirm": "Данная операция может занять некоторое время. Продолжать?",
    "errorSelectProperty": "Выберите хотя бы одно свойство.",
    "errorSelectComponent": "Выберите компонент.",
    "noCombinationsFound": "Сочетаний не найдено"
  }
```

`en`:
```json
  "menu": {
    "file": "File",
    "editDatabase": "Edit Database",
    "settings": "Settings",
    "exit": "Exit",
    "about": "About"
  },
  "app": {
    "resultsFound_one": "Found {{count}} combination",
    "resultsFound_other": "Found {{count}} combinations",
    "resultsDisplayed_one": "Displaying {{count}} combination",
    "resultsDisplayed_other": "Displaying {{count}} combinations",
    "slowOperationConfirm": "This operation may take some time. Continue?",
    "errorSelectProperty": "Select at least one property.",
    "errorSelectComponent": "Select an ingredient.",
    "noCombinationsFound": "No combinations found"
  }
```

`fr`:
```json
  "menu": {
    "file": "Fichier",
    "editDatabase": "Modifier la base",
    "settings": "Paramètres",
    "exit": "Quitter",
    "about": "À propos"
  },
  "app": {
    "resultsFound_one": "{{count}} combinaison trouvée",
    "resultsFound_other": "{{count}} combinaisons trouvées",
    "resultsDisplayed_one": "{{count}} combinaison affichée",
    "resultsDisplayed_other": "{{count}} combinaisons affichées",
    "slowOperationConfirm": "Cette opération peut prendre un certain temps. Continuer ?",
    "errorSelectProperty": "Sélectionnez au moins une propriété.",
    "errorSelectComponent": "Sélectionnez un ingrédient.",
    "noCombinationsFound": "Aucune combinaison trouvée"
  }
```

`de`:
```json
  "menu": {
    "file": "Datei",
    "editDatabase": "Datenbank bearbeiten",
    "settings": "Einstellungen",
    "exit": "Beenden",
    "about": "Über"
  },
  "app": {
    "resultsFound_one": "{{count}} Kombination gefunden",
    "resultsFound_other": "{{count}} Kombinationen gefunden",
    "resultsDisplayed_one": "{{count}} Kombination angezeigt",
    "resultsDisplayed_other": "{{count}} Kombinationen angezeigt",
    "slowOperationConfirm": "Dieser Vorgang kann einige Zeit dauern. Fortfahren?",
    "errorSelectProperty": "Wählen Sie mindestens eine Eigenschaft aus.",
    "errorSelectComponent": "Wählen Sie eine Zutat aus.",
    "noCombinationsFound": "Keine Kombinationen gefunden"
  }
```

`it`:
```json
  "menu": {
    "file": "File",
    "editDatabase": "Modifica database",
    "settings": "Impostazioni",
    "exit": "Esci",
    "about": "Informazioni"
  },
  "app": {
    "resultsFound_one": "Trovata {{count}} combinazione",
    "resultsFound_other": "Trovate {{count}} combinazioni",
    "resultsDisplayed_one": "Visualizzata {{count}} combinazione",
    "resultsDisplayed_other": "Visualizzate {{count}} combinazioni",
    "slowOperationConfirm": "Questa operazione potrebbe richiedere del tempo. Continuare?",
    "errorSelectProperty": "Seleziona almeno una proprietà.",
    "errorSelectComponent": "Seleziona un ingrediente.",
    "noCombinationsFound": "Nessuna combinazione trovata"
  }
```

`es`:
```json
  "menu": {
    "file": "Archivo",
    "editDatabase": "Editar base de datos",
    "settings": "Configuración",
    "exit": "Salir",
    "about": "Acerca de"
  },
  "app": {
    "resultsFound_one": "Se encontró {{count}} combinación",
    "resultsFound_other": "Se encontraron {{count}} combinaciones",
    "resultsDisplayed_one": "Se muestra {{count}} combinación",
    "resultsDisplayed_other": "Se muestran {{count}} combinaciones",
    "slowOperationConfirm": "Esta operación puede tardar algún tiempo. ¿Continuar?",
    "errorSelectProperty": "Selecciona al menos una propiedad.",
    "errorSelectComponent": "Selecciona un ingrediente.",
    "noCombinationsFound": "No se encontraron combinaciones"
  }
```

`pl`:
```json
  "menu": {
    "file": "Plik",
    "editDatabase": "Edytuj bazę",
    "settings": "Ustawienia",
    "exit": "Wyjście",
    "about": "O programie"
  },
  "app": {
    "resultsFound_one": "Znaleziono {{count}} kombinację",
    "resultsFound_few": "Znaleziono {{count}} kombinacje",
    "resultsFound_many": "Znaleziono {{count}} kombinacji",
    "resultsFound_other": "Znaleziono {{count}} kombinacji",
    "resultsDisplayed_one": "Wyświetlono {{count}} kombinację",
    "resultsDisplayed_few": "Wyświetlono {{count}} kombinacje",
    "resultsDisplayed_many": "Wyświetlono {{count}} kombinacji",
    "resultsDisplayed_other": "Wyświetlono {{count}} kombinacji",
    "slowOperationConfirm": "Ta operacja może zająć trochę czasu. Kontynuować?",
    "errorSelectProperty": "Wybierz co najmniej jedną właściwość.",
    "errorSelectComponent": "Wybierz składnik.",
    "noCombinationsFound": "Nie znaleziono kombinacji"
  }
```

`ja`:
```json
  "menu": {
    "file": "ファイル",
    "editDatabase": "データベースを編集",
    "settings": "設定",
    "exit": "終了",
    "about": "このアプリについて"
  },
  "app": {
    "resultsFound_other": "{{count}} 件の組み合わせが見つかりました",
    "resultsDisplayed_other": "{{count}} 件の組み合わせを表示中",
    "slowOperationConfirm": "この操作には時間がかかる場合があります。続行しますか?",
    "errorSelectProperty": "少なくとも1つの特性を選択してください。",
    "errorSelectComponent": "材料を選択してください。",
    "noCombinationsFound": "組み合わせが見つかりませんでした"
  }
```

`zh-Hant`:
```json
  "menu": {
    "file": "檔案",
    "editDatabase": "編輯資料庫",
    "settings": "設定",
    "exit": "結束",
    "about": "關於"
  },
  "app": {
    "resultsFound_other": "找到 {{count}} 個組合",
    "resultsDisplayed_other": "顯示 {{count}} 個組合",
    "slowOperationConfirm": "此操作可能需要一些時間。是否繼續?",
    "errorSelectProperty": "請至少選擇一個屬性。",
    "errorSelectComponent": "請選擇材料。",
    "noCombinationsFound": "未找到組合"
  }
```

- [ ] **Step 2: Подключить `useTranslation` и заменить строки в `App.tsx`**

Импорт (рядом с `import i18n from "./i18n";` из Task 1):
```tsx
import { useTranslation } from "react-i18next";
```

В теле компонента, сразу после сигнатуры `export default function App(...)`:
```tsx
  const { t } = useTranslation();
```

Заменить:
- `useState("Найдено 0 комбинаций")` → `useState(() => t("app.resultsFound", { count: 0 }))`
- В эффекте `[enabledAddons, language]`: `setResultsHeader("Найдено 0 комбинаций")`
  → `setResultsHeader(t("app.resultsFound", { count: 0 }))`
- `confirmThenRun`: `text: "Данная операция может занять некоторое время. Продолжать?"`
  → `text: t("app.slowOperationConfirm")`
- `setPlainResults`: `` setResultsHeader(`Найдено ${items.length} комбинаций`) ``
  → `` setResultsHeader(t("app.resultsFound", { count: items.length })) ``
- `setBlockResults`: `` setResultsHeader(`Отображено ${displayedCount} комбинаций`) ``
  → `` setResultsHeader(t("app.resultsDisplayed", { count: displayedCount })) ``
- `applyComponentFilter`: `"Сочетаний не найдено"` → `t("app.noCombinationsFound")`
- `handleFindCombinations`: `info_("Внимание", "Выберите хотя бы одно свойство.")`
  → `info_(t("common.attention"), t("app.errorSelectProperty"))`
- `handleFindCombinations` catch: `info_("Ошибка", String(e))` → `info_(t("common.error"), String(e))`
- `handleShowProperties`: `info_("Внимание", "Выберите компонент.")`
  → `info_(t("common.attention"), t("app.errorSelectComponent"))`
- `handleShowProperties` catch, `handleFindPairs` catch, `handleFindMaxCombinations`
  catch: `info_("Ошибка", String(e))` → `info_(t("common.error"), String(e))` (3 места)
- `handleFindPairs`/`handleFindMaxCombinations`: `"Сочетаний не найдено"` →
  `t("app.noCombinationsFound")` (2 места, внутри `setBlockResults(found, ...)`)

В JSX (меню):
```tsx
        <Menu shadow="md" width={200}>
          <Menu.Target>
            <button className="menu-bar-button">{t("menu.file")}</button>
          </Menu.Target>
          <Menu.Dropdown>
            <Menu.Item leftSection={<IconPencil size={16} />} onClick={() => setEditorOpen(true)}>
              {t("menu.editDatabase")}
            </Menu.Item>
            <Menu.Item leftSection={<IconSettings size={16} />} onClick={() => setSettingsOpen(true)}>
              {t("menu.settings")}
            </Menu.Item>
            <Menu.Divider />
            <Menu.Item leftSection={<IconDoorExit size={16} />} onClick={handleExit}>
              {t("menu.exit")}
            </Menu.Item>
          </Menu.Dropdown>
        </Menu>
        <Menu shadow="md" width={200}>
          <Menu.Target>
            <button className="menu-bar-button">{t("menu.about")}</button>
          </Menu.Target>
          <Menu.Dropdown>
            <Menu.Item leftSection={<IconHelpCircle size={16} />} onClick={() => setAboutOpen(true)}>
              {t("menu.about")}
            </Menu.Item>
          </Menu.Dropdown>
        </Menu>
```

В JSX (подтверждение операции, ближе к концу файла):
```tsx
      <Modal opened={confirm !== null} onClose={() => setConfirm(null)} title={t("common.confirmTitle")} size="sm">
        <Stack gap="md">
          <Text size="sm">{confirm?.text}</Text>
          <Group justify="flex-end" gap="sm">
            <Button variant="default" onClick={() => setConfirm(null)}>
              {t("common.cancel")}
            </Button>
            <Button
              onClick={() => {
                confirm?.onConfirm();
                setConfirm(null);
              }}
            >
              {t("common.continue")}
            </Button>
          </Group>
        </Stack>
      </Modal>
```

- [ ] **Step 3: Проверить сборку**

```bash
npx tsc -b && npm run build && npm run lint
```

- [ ] **Step 4: Commit**

```bash
git add src/App.tsx src/i18n
git commit -m "feat: translate App.tsx menu and remaining UI strings"
```

---

### Task 4: `EditorModal.tsx`

**Files:**
- Modify: `src/components/EditorModal.tsx`
- Modify: все 9 `src/i18n/locales/<lang>/translation.json`

**Interfaces:**
- Consumes: `common.*` (Task 2, включая `common.propertyLabel` для лейблов
  "Свойство N" и `common.notSelected` для плейсхолдера).
- Produces: не потребляется другими задачами.

- [ ] **Step 1: Добавить `editorModal.*` во все 9 файлов**

`ru`:
```json
  "editorModal": {
    "title": "Редактировать базу",
    "ingredientLabel": "Ингредиент",
    "newButton": "Новый",
    "imageLabel": "Изображение",
    "selectImageButton": "Выбрать файл с изображением",
    "removeImageButton": "Удалить изображение из базы",
    "descriptionLabel": "Описание",
    "descriptionHint": "Корректировки описания ингредиента будут сохранены только для текущего языка",
    "deleteButton": "Удалить",
    "saveButton": "Сохранить",
    "newDialogTitle": "Новый ингредиент",
    "nameLabel": "Название",
    "newDialogLanguageHint": "Ингредиент будет виден только при выбранном языке — {{language}}.",
    "createButton": "Создать",
    "confirmBaseEditText": "Вы уверены, что хотите внести изменения в описание базового ингредиента?",
    "confirmDeleteTitle": "Удаление компонента",
    "confirmDeleteText": "Удалить компонент «{{name}}»? Действие необратимо.",
    "unsavedChangesTitle": "Несохранённые изменения",
    "unsavedChangesText": "Данные не сохранены. Продолжить и потерять изменения?",
    "errorNameRequired": "Введите название ингредиента.",
    "errorNameExists": "Ингредиент «{{name}}» уже существует.",
    "errorSelectAllProperties": "Выберите значение во всех 4 полях «Свойство» (поле {{index}}).",
    "errorDuplicateProperties": "Свойства компонента не должны повторяться.",
    "savedText": "Изменения сохранены.",
    "deletedText": "Компонент удалён."
  }
```

`en`:
```json
  "editorModal": {
    "title": "Edit Database",
    "ingredientLabel": "Ingredient",
    "newButton": "New",
    "imageLabel": "Image",
    "selectImageButton": "Select image file",
    "removeImageButton": "Remove image from database",
    "descriptionLabel": "Description",
    "descriptionHint": "Description edits are saved only for the current language",
    "deleteButton": "Delete",
    "saveButton": "Save",
    "newDialogTitle": "New Ingredient",
    "nameLabel": "Name",
    "newDialogLanguageHint": "The ingredient will only be visible with the selected language — {{language}}.",
    "createButton": "Create",
    "confirmBaseEditText": "Are you sure you want to edit the description of a base-game ingredient?",
    "confirmDeleteTitle": "Delete Ingredient",
    "confirmDeleteText": "Delete ingredient \"{{name}}\"? This action cannot be undone.",
    "unsavedChangesTitle": "Unsaved Changes",
    "unsavedChangesText": "Data is not saved. Continue and lose the changes?",
    "errorNameRequired": "Enter the ingredient's name.",
    "errorNameExists": "Ingredient \"{{name}}\" already exists.",
    "errorSelectAllProperties": "Select a value in all 4 \"Property\" fields (field {{index}}).",
    "errorDuplicateProperties": "The ingredient's properties must not repeat.",
    "savedText": "Changes saved.",
    "deletedText": "Ingredient deleted."
  }
```

`fr`:
```json
  "editorModal": {
    "title": "Modifier la base",
    "ingredientLabel": "Ingrédient",
    "newButton": "Nouveau",
    "imageLabel": "Image",
    "selectImageButton": "Choisir un fichier image",
    "removeImageButton": "Supprimer l'image de la base",
    "descriptionLabel": "Description",
    "descriptionHint": "Les modifications de la description ne sont enregistrées que pour la langue actuelle",
    "deleteButton": "Supprimer",
    "saveButton": "Enregistrer",
    "newDialogTitle": "Nouvel ingrédient",
    "nameLabel": "Nom",
    "newDialogLanguageHint": "L'ingrédient ne sera visible qu'avec la langue sélectionnée — {{language}}.",
    "createButton": "Créer",
    "confirmBaseEditText": "Voulez-vous vraiment modifier la description d'un ingrédient du jeu de base ?",
    "confirmDeleteTitle": "Supprimer l'ingrédient",
    "confirmDeleteText": "Supprimer l'ingrédient « {{name}} » ? Cette action est irréversible.",
    "unsavedChangesTitle": "Modifications non enregistrées",
    "unsavedChangesText": "Les données ne sont pas enregistrées. Continuer et perdre les modifications ?",
    "errorNameRequired": "Saisissez le nom de l'ingrédient.",
    "errorNameExists": "L'ingrédient « {{name}} » existe déjà.",
    "errorSelectAllProperties": "Sélectionnez une valeur dans les 4 champs « Propriété » (champ {{index}}).",
    "errorDuplicateProperties": "Les propriétés de l'ingrédient ne doivent pas se répéter.",
    "savedText": "Modifications enregistrées.",
    "deletedText": "Ingrédient supprimé."
  }
```

`de`:
```json
  "editorModal": {
    "title": "Datenbank bearbeiten",
    "ingredientLabel": "Zutat",
    "newButton": "Neu",
    "imageLabel": "Bild",
    "selectImageButton": "Bilddatei auswählen",
    "removeImageButton": "Bild aus der Datenbank entfernen",
    "descriptionLabel": "Beschreibung",
    "descriptionHint": "Änderungen an der Beschreibung werden nur für die aktuelle Sprache gespeichert",
    "deleteButton": "Löschen",
    "saveButton": "Speichern",
    "newDialogTitle": "Neue Zutat",
    "nameLabel": "Name",
    "newDialogLanguageHint": "Die Zutat ist nur bei der ausgewählten Sprache sichtbar — {{language}}.",
    "createButton": "Erstellen",
    "confirmBaseEditText": "Möchten Sie die Beschreibung einer Basisspiel-Zutat wirklich bearbeiten?",
    "confirmDeleteTitle": "Zutat löschen",
    "confirmDeleteText": "Zutat „{{name}}“ löschen? Diese Aktion kann nicht rückgängig gemacht werden.",
    "unsavedChangesTitle": "Ungespeicherte Änderungen",
    "unsavedChangesText": "Die Daten sind nicht gespeichert. Fortfahren und die Änderungen verlieren?",
    "errorNameRequired": "Geben Sie den Namen der Zutat ein.",
    "errorNameExists": "Die Zutat „{{name}}“ existiert bereits.",
    "errorSelectAllProperties": "Wählen Sie in allen 4 Feldern „Eigenschaft“ einen Wert aus (Feld {{index}}).",
    "errorDuplicateProperties": "Die Eigenschaften der Zutat dürfen sich nicht wiederholen.",
    "savedText": "Änderungen gespeichert.",
    "deletedText": "Zutat gelöscht."
  }
```

`it`:
```json
  "editorModal": {
    "title": "Modifica database",
    "ingredientLabel": "Ingrediente",
    "newButton": "Nuovo",
    "imageLabel": "Immagine",
    "selectImageButton": "Seleziona file immagine",
    "removeImageButton": "Rimuovi l'immagine dal database",
    "descriptionLabel": "Descrizione",
    "descriptionHint": "Le modifiche alla descrizione vengono salvate solo per la lingua corrente",
    "deleteButton": "Elimina",
    "saveButton": "Salva",
    "newDialogTitle": "Nuovo ingrediente",
    "nameLabel": "Nome",
    "newDialogLanguageHint": "L'ingrediente sarà visibile solo con la lingua selezionata — {{language}}.",
    "createButton": "Crea",
    "confirmBaseEditText": "Vuoi davvero modificare la descrizione di un ingrediente del gioco base?",
    "confirmDeleteTitle": "Elimina ingrediente",
    "confirmDeleteText": "Eliminare l'ingrediente «{{name}}»? L'azione è irreversibile.",
    "unsavedChangesTitle": "Modifiche non salvate",
    "unsavedChangesText": "I dati non sono stati salvati. Continuare e perdere le modifiche?",
    "errorNameRequired": "Inserisci il nome dell'ingrediente.",
    "errorNameExists": "L'ingrediente «{{name}}» esiste già.",
    "errorSelectAllProperties": "Seleziona un valore in tutti e 4 i campi «Proprietà» (campo {{index}}).",
    "errorDuplicateProperties": "Le proprietà dell'ingrediente non devono ripetersi.",
    "savedText": "Modifiche salvate.",
    "deletedText": "Ingrediente eliminato."
  }
```

`es`:
```json
  "editorModal": {
    "title": "Editar base de datos",
    "ingredientLabel": "Ingrediente",
    "newButton": "Nuevo",
    "imageLabel": "Imagen",
    "selectImageButton": "Seleccionar archivo de imagen",
    "removeImageButton": "Eliminar imagen de la base de datos",
    "descriptionLabel": "Descripción",
    "descriptionHint": "Los cambios en la descripción se guardan solo para el idioma actual",
    "deleteButton": "Eliminar",
    "saveButton": "Guardar",
    "newDialogTitle": "Nuevo ingrediente",
    "nameLabel": "Nombre",
    "newDialogLanguageHint": "El ingrediente solo será visible con el idioma seleccionado — {{language}}.",
    "createButton": "Crear",
    "confirmBaseEditText": "¿Seguro que quieres modificar la descripción de un ingrediente del juego base?",
    "confirmDeleteTitle": "Eliminar ingrediente",
    "confirmDeleteText": "¿Eliminar el ingrediente «{{name}}»? Esta acción no se puede deshacer.",
    "unsavedChangesTitle": "Cambios sin guardar",
    "unsavedChangesText": "Los datos no están guardados. ¿Continuar y perder los cambios?",
    "errorNameRequired": "Introduce el nombre del ingrediente.",
    "errorNameExists": "El ingrediente «{{name}}» ya existe.",
    "errorSelectAllProperties": "Selecciona un valor en los 4 campos «Propiedad» (campo {{index}}).",
    "errorDuplicateProperties": "Las propiedades del ingrediente no deben repetirse.",
    "savedText": "Cambios guardados.",
    "deletedText": "Ingrediente eliminado."
  }
```

`pl`:
```json
  "editorModal": {
    "title": "Edytuj bazę",
    "ingredientLabel": "Składnik",
    "newButton": "Nowy",
    "imageLabel": "Obraz",
    "selectImageButton": "Wybierz plik obrazu",
    "removeImageButton": "Usuń obraz z bazy danych",
    "descriptionLabel": "Opis",
    "descriptionHint": "Zmiany w opisie są zapisywane tylko dla bieżącego języka",
    "deleteButton": "Usuń",
    "saveButton": "Zapisz",
    "newDialogTitle": "Nowy składnik",
    "nameLabel": "Nazwa",
    "newDialogLanguageHint": "Składnik będzie widoczny tylko przy wybranym języku — {{language}}.",
    "createButton": "Utwórz",
    "confirmBaseEditText": "Czy na pewno chcesz zmienić opis podstawowego składnika?",
    "confirmDeleteTitle": "Usuwanie składnika",
    "confirmDeleteText": "Usunąć składnik „{{name}}”? Tej operacji nie można cofnąć.",
    "unsavedChangesTitle": "Niezapisane zmiany",
    "unsavedChangesText": "Dane nie zostały zapisane. Kontynuować i utracić zmiany?",
    "errorNameRequired": "Wpisz nazwę składnika.",
    "errorNameExists": "Składnik „{{name}}” już istnieje.",
    "errorSelectAllProperties": "Wybierz wartość we wszystkich 4 polach „Właściwość” (pole {{index}}).",
    "errorDuplicateProperties": "Właściwości składnika nie mogą się powtarzać.",
    "savedText": "Zmiany zapisane.",
    "deletedText": "Składnik usunięty."
  }
```

`ja`:
```json
  "editorModal": {
    "title": "データベースを編集",
    "ingredientLabel": "材料",
    "newButton": "新規",
    "imageLabel": "画像",
    "selectImageButton": "画像ファイルを選択",
    "removeImageButton": "データベースから画像を削除",
    "descriptionLabel": "説明",
    "descriptionHint": "説明の変更は現在の言語にのみ保存されます",
    "deleteButton": "削除",
    "saveButton": "保存",
    "newDialogTitle": "新しい材料",
    "nameLabel": "名前",
    "newDialogLanguageHint": "この材料は選択した言語（{{language}}）でのみ表示されます。",
    "createButton": "作成",
    "confirmBaseEditText": "ベースゲームの材料の説明を編集してもよろしいですか?",
    "confirmDeleteTitle": "材料の削除",
    "confirmDeleteText": "材料「{{name}}」を削除しますか?この操作は元に戻せません。",
    "unsavedChangesTitle": "未保存の変更",
    "unsavedChangesText": "データが保存されていません。続行して変更を破棄しますか?",
    "errorNameRequired": "材料の名前を入力してください。",
    "errorNameExists": "材料「{{name}}」はすでに存在します。",
    "errorSelectAllProperties": "4つの「特性」フィールドすべてに値を選択してください（フィールド{{index}}）。",
    "errorDuplicateProperties": "材料の特性を重複させることはできません。",
    "savedText": "変更を保存しました。",
    "deletedText": "材料を削除しました。"
  }
```

`zh-Hant`:
```json
  "editorModal": {
    "title": "編輯資料庫",
    "ingredientLabel": "材料",
    "newButton": "新增",
    "imageLabel": "圖片",
    "selectImageButton": "選擇圖片檔案",
    "removeImageButton": "從資料庫刪除圖片",
    "descriptionLabel": "說明",
    "descriptionHint": "說明的修改僅會儲存於目前語言",
    "deleteButton": "刪除",
    "saveButton": "儲存",
    "newDialogTitle": "新增材料",
    "nameLabel": "名稱",
    "newDialogLanguageHint": "此材料僅在所選語言（{{language}}）下可見。",
    "createButton": "建立",
    "confirmBaseEditText": "確定要編輯基礎遊戲材料的說明嗎?",
    "confirmDeleteTitle": "刪除材料",
    "confirmDeleteText": "要刪除材料「{{name}}」嗎?此操作無法復原。",
    "unsavedChangesTitle": "未儲存的變更",
    "unsavedChangesText": "資料尚未儲存。是否繼續並放棄變更?",
    "errorNameRequired": "請輸入材料名稱。",
    "errorNameExists": "材料「{{name}}」已存在。",
    "errorSelectAllProperties": "請在全部 4 個「屬性」欄位中選擇數值（欄位 {{index}}）。",
    "errorDuplicateProperties": "材料的屬性不可重複。",
    "savedText": "變更已儲存。",
    "deletedText": "材料已刪除。"
  }
```

- [ ] **Step 2: Подключить `useTranslation` в `EditorModal.tsx`**

Импорт:
```tsx
import { useTranslation } from "react-i18next";
```

В теле компонента, после сигнатуры `export function EditorModal(...)`:
```tsx
  const { t } = useTranslation();
```

- [ ] **Step 3: Заменить строки внутри функций**

- `confirmNewName`: `setInfo({ title: "Ошибка", text: "Введите название ингредиента." })`
  → `setInfo({ title: t("common.error"), text: t("editorModal.errorNameRequired") })`
- `` setInfo({ title: "Ошибка", text: `Ингредиент «${name}» уже существует.` }) ``
  → `setInfo({ title: t("common.error"), text: t("editorModal.errorNameExists", { name }) })`
- `handleSave`: `` setInfo({ title: "Ошибка", text: `Выберите значение во всех 4 полях «Свойство» (поле ${i + 1}).` }) ``
  → `setInfo({ title: t("common.error"), text: t("editorModal.errorSelectAllProperties", { index: i + 1 }) })`
- `setInfo({ title: "Ошибка", text: "Свойства компонента не должны повторяться." })`
  → `setInfo({ title: t("common.error"), text: t("editorModal.errorDuplicateProperties") })`
- `performSave`: `setInfo({ title: "Готово", text: "Изменения сохранены." })`
  → `setInfo({ title: t("common.done"), text: t("editorModal.savedText") })`
- `performSave` catch: `setInfo({ title: "Ошибка", text: String(e) })` → `setInfo({ title: t("common.error"), text: String(e) })`
- `handleDelete`: `setInfo({ title: "Готово", text: "Компонент удалён." })`
  → `setInfo({ title: t("common.done"), text: t("editorModal.deletedText") })`
- `handleDelete` catch: `setInfo({ title: "Ошибка", text: String(e) })` → `setInfo({ title: t("common.error"), text: String(e) })`

- [ ] **Step 4: Заменить JSX**

Заголовок модалки:
```tsx
    <Modal opened={opened} onClose={() => requestAction({ kind: "close" })} title={t("editorModal.title")} size="xl">
```

Блок "Ингредиент" + кнопка "Новый":
```tsx
        <div>
          <Text size="sm" fw={700} mb={2}>
            {t("editorModal.ingredientLabel")}
          </Text>
          <Group wrap="nowrap" gap="xs" align="flex-start">
            {isNew ? (
              <TextInput flex={1} value={loadedName} disabled />
            ) : (
              <Select
                flex={1}
                data={names.map((n) => ({ value: String(n.id), label: n.name }))}
                value={loadedId !== null ? String(loadedId) : null}
                onChange={(v) => {
                  if (v === null) return;
                  const id = Number(v);
                  const found = names.find((n) => n.id === id);
                  if (found) requestAction({ kind: "load", id: found.id, name: found.name });
                }}
                searchable
                clearable={false}
                comboboxProps={{ withinPortal: true }}
              />
            )}
            <Button variant="default" onClick={() => requestAction({ kind: "new" })}>
              {t("editorModal.newButton")}
            </Button>
          </Group>
        </div>
```

Сетка свойств (плейсхолдер и лейбл — `common.*`):
```tsx
        <SimpleGrid cols={2}>
          {[0, 1, 2, 3].map((i) => (
            <Select
              key={i}
              label={t("common.propertyLabel", { index: i + 1 })}
              placeholder={t("common.notSelected")}
              data={allProperties.map((p) => ({ value: String(p.id), label: p.name }))}
              value={propSelects[i] !== null ? String(propSelects[i]) : null}
              disabled={!editable}
              onChange={(v) => {
                const next = [...propSelects] as PropIds;
                next[i] = v !== null ? Number(v) : null;
                setPropSelects(next);
                recomputeDirty({ props: next });
              }}
              searchable
              clearable
              comboboxProps={{ withinPortal: true }}
            />
          ))}
        </SimpleGrid>
```

Блок "Изображение":
```tsx
        <div>
          <Text size="sm" fw={700} mb={4}>
            {t("editorModal.imageLabel")}
          </Text>
          <Group align="center" wrap="nowrap">
            <div
              style={{
                width: PREVIEW_BOX,
                height: PREVIEW_BOX,
                minWidth: PREVIEW_BOX,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                border: "1px solid var(--mantine-color-gray-4)",
                borderRadius: 6,
              }}
            >
              {previewSrc ? (
                <Image src={previewSrc} w={previewSize} h={previewSize} fit="contain" />
              ) : (
                <Text c="dimmed">{t("common.noImage")}</Text>
              )}
            </div>
            <Stack flex={1} gap={4}>
              <Button variant="light" onClick={pickImage}>
                {t("editorModal.selectImageButton")}
              </Button>
              <Button variant="light" color="red" disabled={!imageBase64} onClick={clearImage}>
                {t("editorModal.removeImageButton")}
              </Button>
              {imageFileName && (
                <Text size="xs" c="dimmed">
                  {imageFileName}
                </Text>
              )}
            </Stack>
          </Group>
        </div>
```

Блок "Описание":
```tsx
        <div>
          <Group gap={6} align="baseline">
            <Text size="sm" fw={700}>
              {t("editorModal.descriptionLabel")}
            </Text>
            <HintIcon label={t("editorModal.descriptionHint")} />
          </Group>
          <Textarea
            key={loadedName}
            styles={{ input: { minHeight: DESCRIPTION_HEIGHT } }}
            ref={descriptionRef}
            defaultValue={description}
            onChange={() => {
              setDescriptionTouched(true);
              setDirty(true);
            }}
          />
        </div>
```

Нижняя панель кнопок:
```tsx
        <Group justify="space-between" mt="xs">
          <Group>
            <Button
              variant="default"
              color="red"
              disabled={!loadedName || isNew || !editable}
              onClick={() => setConfirmDelete(true)}
            >
              {t("editorModal.deleteButton")}
            </Button>
          </Group>
          <Group>
            <Button variant="default" onClick={() => requestAction({ kind: "close" })}>
              {t("common.close")}
            </Button>
            <Button disabled={!dirty} onClick={handleSave}>
              {t("editorModal.saveButton")}
            </Button>
          </Group>
        </Group>
```

Диалог "Новый ингредиент":
```tsx
      <Modal opened={newDialogOpen} onClose={() => setNewDialogOpen(false)} title={t("editorModal.newDialogTitle")} size="sm">
        <TextInput
          key={newDialogGeneration}
          label={t("editorModal.nameLabel")}
          ref={newNameRef}
          defaultValue=""
          data-autofocus
        />
        <Text size="xs" c="dimmed" mt={4}>
          {t("editorModal.newDialogLanguageHint", {
            language: LANGUAGE_OPTIONS.find((l) => l.value === lang)?.label ?? lang,
          })}
        </Text>
        <Group justify="flex-end" mt="md">
          <Button variant="default" onClick={() => setNewDialogOpen(false)}>
            {t("common.cancel")}
          </Button>
          <Button onClick={confirmNewName}>{t("editorModal.createButton")}</Button>
        </Group>
      </Modal>
```

Диалог подтверждения правки базового ингредиента:
```tsx
      <Modal opened={confirmBaseEdit} onClose={() => setConfirmBaseEdit(false)} title={t("common.confirmTitle")} size="sm">
        <Text size="sm" mb="md">
          {t("editorModal.confirmBaseEditText")}
        </Text>
        <Group justify="flex-end">
          <Button variant="default" onClick={() => setConfirmBaseEdit(false)}>
            {t("common.cancel")}
          </Button>
          <Button
            onClick={() => {
              setConfirmBaseEdit(false);
              performSave();
            }}
          >
            {t("common.continue")}
          </Button>
        </Group>
      </Modal>
```

Диалог удаления:
```tsx
      <Modal opened={confirmDelete} onClose={() => setConfirmDelete(false)} title={t("editorModal.confirmDeleteTitle")} size="sm">
        <Text size="sm" mb="md">
          {t("editorModal.confirmDeleteText", { name: loadedName })}
        </Text>
        <Group justify="flex-end">
          <Button variant="default" onClick={() => setConfirmDelete(false)}>
            {t("common.cancel")}
          </Button>
          <Button color="red" onClick={handleDelete}>
            {t("editorModal.deleteButton")}
          </Button>
        </Group>
      </Modal>
```

Диалог несохранённых изменений:
```tsx
      <Modal
        opened={pendingAction !== null}
        onClose={() => setPendingAction(null)}
        title={t("editorModal.unsavedChangesTitle")}
        size="sm"
      >
        <Text size="sm" mb="md">
          {t("editorModal.unsavedChangesText")}
        </Text>
        <Group justify="flex-end">
          <Button variant="default" onClick={() => setPendingAction(null)}>
            {t("common.cancel")}
          </Button>
          <Button
            color="red"
            onClick={() => {
              const action = pendingAction;
              setPendingAction(null);
              if (action) runAction(action);
            }}
          >
            {t("common.continue")}
          </Button>
        </Group>
      </Modal>
```

- [ ] **Step 5: Проверить сборку**

```bash
npx tsc -b && npm run build && npm run lint
```

- [ ] **Step 6: Commit**

```bash
git add src/components/EditorModal.tsx src/i18n
git commit -m "feat: translate EditorModal UI strings"
```

---

### Task 5: `SettingsModal.tsx`

**Files:**
- Modify: `src/components/SettingsModal.tsx`
- Modify: все 9 `src/i18n/locales/<lang>/translation.json`

**Interfaces:**
- Consumes: `common.*` (Task 2). НЕ трогает `src/lib/addons.ts` — перевод
  5 переводимых `AddonId` добавляется в новом namespace `addons.*` внутри
  локалей и резолвится локальной функцией внутри `SettingsModal.tsx`, файл
  `addons.ts` остаётся как есть (это его исходно заложенное назначение —
  см. Global Constraints).
- Produces: не потребляется другими задачами.

- [ ] **Step 1: Добавить `settingsModal.*` и `addons.*` во все 9 файлов**

`ru`:
```json
  "settingsModal": {
    "title": "Настройки",
    "languageLabel": "Язык",
    "scaleLabel": "Масштаб",
    "scaleSmall": "Мелкий",
    "scaleNormal": "Нормальный",
    "scaleLarge": "Крупный",
    "themeLabel": "Цветовая тема",
    "themeAuto": "Системная",
    "themeLight": "Светлая",
    "themeDark": "Тёмная",
    "maxCombinationsLabel": "Макс. кол-во сочетаний",
    "maxCombinationsHint": "Количество комбинаций ингредиентов, отображаемое при нажатии на кнопки «Парные сочетания» и «Тройные сочетания»",
    "addonsLabel": "Дополнения к игре",
    "applyButton": "Применить",
    "languageWarningText": "У вас есть ингредиенты, добавленные вручную — они видны только на языке, на котором были созданы. После смены языка они пропадут из списков (ничего не удаляется — снова появятся, если вернуться на прежний язык). Продолжить?"
  },
  "addons": {
    "baseGame": "Базовый набор",
    "fishing": "Рыбалка",
    "saintsAndSeducers": "Святые и соблазнители",
    "plagueOfTheDead": "Чума мертвецов",
    "userAdded": "Добавлено пользователем"
  }
```

`en`:
```json
  "settingsModal": {
    "title": "Settings",
    "languageLabel": "Language",
    "scaleLabel": "Scale",
    "scaleSmall": "Small",
    "scaleNormal": "Normal",
    "scaleLarge": "Large",
    "themeLabel": "Color Theme",
    "themeAuto": "System",
    "themeLight": "Light",
    "themeDark": "Dark",
    "maxCombinationsLabel": "Max. Combinations",
    "maxCombinationsHint": "The number of ingredient combinations shown when clicking the \"Pair Combinations\" and \"Triple Combinations\" buttons",
    "addonsLabel": "Game Add-ons",
    "applyButton": "Apply",
    "languageWarningText": "You have manually added ingredients — they're only visible in the language they were created in. After changing the language, they'll disappear from the lists (nothing is deleted — they'll reappear if you switch back to the previous language). Continue?"
  },
  "addons": {
    "baseGame": "Base Game",
    "fishing": "Fishing",
    "saintsAndSeducers": "Saints & Seducers",
    "plagueOfTheDead": "Plague of the Dead",
    "userAdded": "User-added"
  }
```

`fr`:
```json
  "settingsModal": {
    "title": "Paramètres",
    "languageLabel": "Langue",
    "scaleLabel": "Échelle",
    "scaleSmall": "Petite",
    "scaleNormal": "Normale",
    "scaleLarge": "Grande",
    "themeLabel": "Thème de couleur",
    "themeAuto": "Système",
    "themeLight": "Clair",
    "themeDark": "Sombre",
    "maxCombinationsLabel": "Nb max. de combinaisons",
    "maxCombinationsHint": "Le nombre de combinaisons d'ingrédients affiché en cliquant sur les boutons « Combinaisons par paires » et « Combinaisons par trois »",
    "addonsLabel": "Extensions du jeu",
    "applyButton": "Appliquer",
    "languageWarningText": "Vous avez des ingrédients ajoutés manuellement — ils ne sont visibles que dans la langue dans laquelle ils ont été créés. Après le changement de langue, ils disparaîtront des listes (rien n'est supprimé — ils réapparaîtront si vous revenez à la langue précédente). Continuer ?"
  },
  "addons": {
    "baseGame": "Jeu de base",
    "fishing": "Pêche",
    "saintsAndSeducers": "Saints et Séducteurs",
    "plagueOfTheDead": "La Peste des morts",
    "userAdded": "Ajouté par l'utilisateur"
  }
```

`de`:
```json
  "settingsModal": {
    "title": "Einstellungen",
    "languageLabel": "Sprache",
    "scaleLabel": "Skalierung",
    "scaleSmall": "Klein",
    "scaleNormal": "Normal",
    "scaleLarge": "Groß",
    "themeLabel": "Farbschema",
    "themeAuto": "System",
    "themeLight": "Hell",
    "themeDark": "Dunkel",
    "maxCombinationsLabel": "Max. Kombinationen",
    "maxCombinationsHint": "Die Anzahl der Zutatenkombinationen, die beim Klicken auf die Schaltflächen „Paar-Kombinationen“ und „Dreier-Kombinationen“ angezeigt wird",
    "addonsLabel": "Spiel-Erweiterungen",
    "applyButton": "Übernehmen",
    "languageWarningText": "Sie haben manuell hinzugefügte Zutaten — sie sind nur in der Sprache sichtbar, in der sie erstellt wurden. Nach dem Sprachwechsel verschwinden sie aus den Listen (nichts wird gelöscht — sie erscheinen wieder, wenn Sie zur vorherigen Sprache zurückwechseln). Fortfahren?"
  },
  "addons": {
    "baseGame": "Basisspiel",
    "fishing": "Angeln",
    "saintsAndSeducers": "Heilige und Verführer",
    "plagueOfTheDead": "Seuche der Toten",
    "userAdded": "Vom Benutzer hinzugefügt"
  }
```

`it`:
```json
  "settingsModal": {
    "title": "Impostazioni",
    "languageLabel": "Lingua",
    "scaleLabel": "Scala",
    "scaleSmall": "Piccola",
    "scaleNormal": "Normale",
    "scaleLarge": "Grande",
    "themeLabel": "Tema colore",
    "themeAuto": "Sistema",
    "themeLight": "Chiaro",
    "themeDark": "Scuro",
    "maxCombinationsLabel": "N. max combinazioni",
    "maxCombinationsHint": "Il numero di combinazioni di ingredienti mostrato cliccando sui pulsanti «Combinazioni in coppia» e «Combinazioni in tripletta»",
    "addonsLabel": "Espansioni di gioco",
    "applyButton": "Applica",
    "languageWarningText": "Hai ingredienti aggiunti manualmente — sono visibili solo nella lingua in cui sono stati creati. Dopo il cambio di lingua, scompariranno dagli elenchi (nulla viene eliminato — riappariranno se torni alla lingua precedente). Continuare?"
  },
  "addons": {
    "baseGame": "Gioco base",
    "fishing": "Pesca",
    "saintsAndSeducers": "Santi e Seduttori",
    "plagueOfTheDead": "Peste dei morti",
    "userAdded": "Aggiunto dall'utente"
  }
```

`es`:
```json
  "settingsModal": {
    "title": "Configuración",
    "languageLabel": "Idioma",
    "scaleLabel": "Escala",
    "scaleSmall": "Pequeña",
    "scaleNormal": "Normal",
    "scaleLarge": "Grande",
    "themeLabel": "Tema de color",
    "themeAuto": "Sistema",
    "themeLight": "Claro",
    "themeDark": "Oscuro",
    "maxCombinationsLabel": "N.º máx. de combinaciones",
    "maxCombinationsHint": "La cantidad de combinaciones de ingredientes que se muestra al pulsar los botones «Combinaciones por pares» y «Combinaciones por tríos»",
    "addonsLabel": "Complementos del juego",
    "applyButton": "Aplicar",
    "languageWarningText": "Tienes ingredientes añadidos manualmente — solo son visibles en el idioma en que se crearon. Tras cambiar el idioma, desaparecerán de las listas (no se elimina nada — reaparecerán si vuelves al idioma anterior). ¿Continuar?"
  },
  "addons": {
    "baseGame": "Juego base",
    "fishing": "Pesca",
    "saintsAndSeducers": "Santos y Seductores",
    "plagueOfTheDead": "Peste de los muertos",
    "userAdded": "Añadido por el usuario"
  }
```

`pl`:
```json
  "settingsModal": {
    "title": "Ustawienia",
    "languageLabel": "Język",
    "scaleLabel": "Skala",
    "scaleSmall": "Mała",
    "scaleNormal": "Normalna",
    "scaleLarge": "Duża",
    "themeLabel": "Motyw kolorystyczny",
    "themeAuto": "Systemowy",
    "themeLight": "Jasny",
    "themeDark": "Ciemny",
    "maxCombinationsLabel": "Maks. liczba kombinacji",
    "maxCombinationsHint": "Liczba kombinacji składników wyświetlana po kliknięciu przycisków „Kombinacje par” i „Kombinacje trójek”",
    "addonsLabel": "Dodatki do gry",
    "applyButton": "Zastosuj",
    "languageWarningText": "Masz ręcznie dodane składniki — są widoczne tylko w języku, w którym zostały utworzone. Po zmianie języka znikną z list (nic nie zostanie usunięte — pojawią się ponownie po powrocie do poprzedniego języka). Kontynuować?"
  },
  "addons": {
    "baseGame": "Podstawowa gra",
    "fishing": "Wędkarstwo",
    "saintsAndSeducers": "Święci i Uwodziciele",
    "plagueOfTheDead": "Zaraza umarłych",
    "userAdded": "Dodane przez użytkownika"
  }
```

`ja`:
```json
  "settingsModal": {
    "title": "設定",
    "languageLabel": "言語",
    "scaleLabel": "スケール",
    "scaleSmall": "小",
    "scaleNormal": "標準",
    "scaleLarge": "大",
    "themeLabel": "カラーテーマ",
    "themeAuto": "システム",
    "themeLight": "ライト",
    "themeDark": "ダーク",
    "maxCombinationsLabel": "最大組み合わせ数",
    "maxCombinationsHint": "「ペア組み合わせ」および「トリプル組み合わせ」ボタンをクリックしたときに表示される材料の組み合わせ数",
    "addonsLabel": "ゲーム追加コンテンツ",
    "applyButton": "適用",
    "languageWarningText": "手動で追加した材料があります。これらは作成時の言語でのみ表示されます。言語を変更すると一覧から消えます（削除はされません。以前の言語に戻すと再表示されます）。続行しますか?"
  },
  "addons": {
    "baseGame": "ベースゲーム",
    "fishing": "釣り",
    "saintsAndSeducers": "聖人と誘惑者",
    "plagueOfTheDead": "死者の疫病",
    "userAdded": "ユーザー追加"
  }
```

`zh-Hant`:
```json
  "settingsModal": {
    "title": "設定",
    "languageLabel": "語言",
    "scaleLabel": "縮放",
    "scaleSmall": "小",
    "scaleNormal": "正常",
    "scaleLarge": "大",
    "themeLabel": "配色主題",
    "themeAuto": "系統",
    "themeLight": "淺色",
    "themeDark": "深色",
    "maxCombinationsLabel": "最大組合數",
    "maxCombinationsHint": "按下「配對組合」和「三重組合」按鈕時顯示的材料組合數量",
    "addonsLabel": "遊戲追加內容",
    "applyButton": "套用",
    "languageWarningText": "您有手動新增的材料——它們僅在建立時所用的語言下可見。變更語言後，它們將從清單中消失（不會被刪除，切換回原語言後會重新出現）。是否繼續?"
  },
  "addons": {
    "baseGame": "基礎遊戲",
    "fishing": "釣魚",
    "saintsAndSeducers": "聖徒與誘惑者",
    "plagueOfTheDead": "死者瘟疫",
    "userAdded": "使用者新增"
  }
```

- [ ] **Step 2: Подключить `useTranslation`, добавить резолвер лейблов
      дополнений (не трогая `addons.ts`)**

Импорт:
```tsx
import { useTranslation } from "react-i18next";
```

В теле компонента, после деструктуризации пропсов:
```tsx
  const { t } = useTranslation();
```

Рядом с константами `SCALE_OPTIONS`/`THEME_OPTIONS` (внутренние значения не
менять, см. Global Constraints) добавить резолвер для 5 переводимых
дополнений — остальные 4 (`dawnguard`/`dragonborn`/`hearthfire`/
`rare_curios`) продолжают браться из уже существующего `ADDON_LABELS`:
```tsx
const TRANSLATABLE_ADDON_KEYS: Partial<Record<AddonId, string>> = {
  base_game: "addons.baseGame",
  fishing: "addons.fishing",
  saints_and_seducers: "addons.saintsAndSeducers",
  plague_of_the_dead: "addons.plagueOfTheDead",
  user_added: "addons.userAdded",
};
```

Внутри компонента, рядом с `const { colorScheme, setColorScheme } = useMantineColorScheme();`:
```tsx
  function addonLabel(id: AddonId): string {
    const key = TRANSLATABLE_ADDON_KEYS[id];
    return key ? t(key) : ADDON_LABELS[id];
  }
```

- [ ] **Step 3: Заменить JSX — лейблы `Select`/`Radio`, оставляя `value` как есть**

Заголовок модалки:
```tsx
      title={t("settingsModal.title")}
```

Левая колонка (Язык/Масштаб/Тема/Макс. кол-во сочетаний):
```tsx
              <Select
                label={t("settingsModal.languageLabel")}
                data={LANGUAGE_OPTIONS}
                value={language}
                onChange={(v) => setLanguage(v ?? language)}
                allowDeselect={false}
                comboboxProps={{ withinPortal: true }}
              />
              <Select
                label={t("settingsModal.scaleLabel")}
                data={SCALE_OPTIONS.map((v) => ({
                  value: v,
                  label:
                    v === "Мелкий"
                      ? t("settingsModal.scaleSmall")
                      : v === "Крупный"
                        ? t("settingsModal.scaleLarge")
                        : t("settingsModal.scaleNormal"),
                }))}
                value={scale}
                onChange={(v) => setScale((v as AppScaleName | null) ?? scale)}
                allowDeselect={false}
                comboboxProps={{ withinPortal: true }}
              />
              <Select
                label={t("settingsModal.themeLabel")}
                data={THEME_OPTIONS.map((v) => ({
                  value: v,
                  label:
                    v === SKYRIM_LABEL
                      ? SKYRIM_LABEL
                      : v === "Светлая"
                        ? t("settingsModal.themeLight")
                        : v === "Тёмная"
                          ? t("settingsModal.themeDark")
                          : t("settingsModal.themeAuto"),
                }))}
                value={theme}
                onChange={(v) => setTheme(v ?? theme)}
                allowDeselect={false}
                comboboxProps={{ withinPortal: true }}
              />
              <TextInput
                label={
                  <Group gap={6} align="center">
                    <span>{t("settingsModal.maxCombinationsLabel")}</span>
                    <HintIcon label={t("settingsModal.maxCombinationsHint")} />
                  </Group>
                }
                value={maxCombinationsInput}
                onChange={(e) => setMaxCombinationsInput(e.currentTarget.value.replace(/\D/g, ""))}
                inputMode="numeric"
              />
```

Правая колонка (заголовок + чекбоксы дополнений):
```tsx
            <Text size="xs" fw={700} p="xs" pb={4}>
              {t("settingsModal.addonsLabel")}
            </Text>
            <ScrollArea flex={1} px="xs">
              <Stack gap={6}>
                {ADDON_CHECKBOX_IDS.map((id) => (
                  <Checkbox
                    key={id}
                    size="xs"
                    color={id === "base_game" ? "yellow" : undefined}
                    label={addonLabel(id)}
                    checked={checkedAddons.includes(id)}
                    onChange={(e) => {
                      const checked = e.currentTarget.checked;
                      setCheckedAddons((prev) =>
                        checked ? [...prev, id] : prev.filter((a) => a !== id),
                      );
                    }}
                  />
                ))}
              </Stack>
            </ScrollArea>
```

Нижняя панель:
```tsx
          <Button variant="default" onClick={onClose}>
            {t("common.cancel")}
          </Button>
          <Button onClick={handleApply}>{t("settingsModal.applyButton")}</Button>
```

Диалог предупреждения о смене языка:
```tsx
      <Modal
        opened={languageWarningOpen}
        onClose={() => setLanguageWarningOpen(false)}
        title={t("common.confirmTitle")}
        size="sm"
      >
        <Text size="sm" mb="md">
          {t("settingsModal.languageWarningText")}
        </Text>
        <Group justify="flex-end">
          <Button variant="default" onClick={() => setLanguageWarningOpen(false)}>
            {t("common.cancel")}
          </Button>
          <Button
            onClick={() => {
              setLanguageWarningOpen(false);
              applyAll();
            }}
          >
            {t("common.continue")}
          </Button>
        </Group>
      </Modal>
```

- [ ] **Step 4: Проверить сборку**

```bash
npx tsc -b && npm run build && npm run lint
```

- [ ] **Step 5: Commit**

```bash
git add src/components/SettingsModal.tsx src/i18n
git commit -m "feat: translate SettingsModal UI strings"
```

---

### Task 6: `ControlPanel.tsx`

**Files:**
- Modify: `src/components/ControlPanel.tsx`
- Modify: все 9 `src/i18n/locales/<lang>/translation.json`

**Interfaces:**
- Consumes: `common.propertyLabel`, `common.notSelected` (Task 2).
- Produces: не потребляется другими задачами.

- [ ] **Step 1: Добавить `controlPanel.*` во все 9 файлов**

`ru`:
```json
  "controlPanel": {
    "searchTitle": "Поиск сочетаний",
    "filterTypeLabel": "Тип свойств",
    "filterAll": "Все",
    "filterBuff": "Улучшения",
    "filterPoison": "Яды",
    "findCombinationsButton": "Найти сочетания",
    "ingredientCountLabel": "Ингредиент ({{count}})",
    "showPropertiesButton": "Показать свойства",
    "pairsButton": "Парные сочетания",
    "pairsHint": "Вывести сочетания двух ингредиентов, дающих максимальное количество эффектов. (Учитывает переключатель «Все»/«Улучшения»/«Яды».)",
    "triplesButton": "Тройные сочетания",
    "triplesHint": "Вывести сочетания трёх ингредиентов, дающих максимальное количество эффектов. (Учитывает переключатель «Все»/«Улучшения»/«Яды».)"
  }
```

`en`:
```json
  "controlPanel": {
    "searchTitle": "Search Combinations",
    "filterTypeLabel": "Property Type",
    "filterAll": "All",
    "filterBuff": "Buffs",
    "filterPoison": "Poisons",
    "findCombinationsButton": "Find Combinations",
    "ingredientCountLabel": "Ingredient ({{count}})",
    "showPropertiesButton": "Show Properties",
    "pairsButton": "Pair Combinations",
    "pairsHint": "Show combinations of two ingredients that give the maximum number of effects. (Respects the \"All\"/\"Buffs\"/\"Poisons\" filter.)",
    "triplesButton": "Triple Combinations",
    "triplesHint": "Show combinations of three ingredients that give the maximum number of effects. (Respects the \"All\"/\"Buffs\"/\"Poisons\" filter.)"
  }
```

`fr`:
```json
  "controlPanel": {
    "searchTitle": "Rechercher des combinaisons",
    "filterTypeLabel": "Type de propriété",
    "filterAll": "Toutes",
    "filterBuff": "Améliorations",
    "filterPoison": "Poisons",
    "findCombinationsButton": "Trouver des combinaisons",
    "ingredientCountLabel": "Ingrédient ({{count}})",
    "showPropertiesButton": "Afficher les propriétés",
    "pairsButton": "Combinaisons par paires",
    "pairsHint": "Afficher les combinaisons de deux ingrédients donnant le plus grand nombre d'effets. (Respecte le filtre « Toutes »/« Améliorations »/« Poisons ».)",
    "triplesButton": "Combinaisons par trois",
    "triplesHint": "Afficher les combinaisons de trois ingrédients donnant le plus grand nombre d'effets. (Respecte le filtre « Toutes »/« Améliorations »/« Poisons ».)"
  }
```

`de`:
```json
  "controlPanel": {
    "searchTitle": "Kombinationen suchen",
    "filterTypeLabel": "Eigenschaftstyp",
    "filterAll": "Alle",
    "filterBuff": "Verbesserungen",
    "filterPoison": "Gifte",
    "findCombinationsButton": "Kombinationen finden",
    "ingredientCountLabel": "Zutat ({{count}})",
    "showPropertiesButton": "Eigenschaften anzeigen",
    "pairsButton": "Paar-Kombinationen",
    "pairsHint": "Zeigt Kombinationen aus zwei Zutaten, die die maximale Anzahl an Effekten ergeben. (Berücksichtigt den Filter „Alle“/„Verbesserungen“/„Gifte“.)",
    "triplesButton": "Dreier-Kombinationen",
    "triplesHint": "Zeigt Kombinationen aus drei Zutaten, die die maximale Anzahl an Effekten ergeben. (Berücksichtigt den Filter „Alle“/„Verbesserungen“/„Gifte“.)"
  }
```

`it`:
```json
  "controlPanel": {
    "searchTitle": "Cerca combinazioni",
    "filterTypeLabel": "Tipo di proprietà",
    "filterAll": "Tutte",
    "filterBuff": "Miglioramenti",
    "filterPoison": "Veleni",
    "findCombinationsButton": "Trova combinazioni",
    "ingredientCountLabel": "Ingrediente ({{count}})",
    "showPropertiesButton": "Mostra proprietà",
    "pairsButton": "Combinazioni in coppia",
    "pairsHint": "Mostra le combinazioni di due ingredienti che danno il numero massimo di effetti. (Rispetta il filtro «Tutte»/«Miglioramenti»/«Veleni».)",
    "triplesButton": "Combinazioni in tripletta",
    "triplesHint": "Mostra le combinazioni di tre ingredienti che danno il numero massimo di effetti. (Rispetta il filtro «Tutte»/«Miglioramenti»/«Veleni».)"
  }
```

`es`:
```json
  "controlPanel": {
    "searchTitle": "Buscar combinaciones",
    "filterTypeLabel": "Tipo de propiedad",
    "filterAll": "Todas",
    "filterBuff": "Mejoras",
    "filterPoison": "Venenos",
    "findCombinationsButton": "Buscar combinaciones",
    "ingredientCountLabel": "Ingrediente ({{count}})",
    "showPropertiesButton": "Mostrar propiedades",
    "pairsButton": "Combinaciones por pares",
    "pairsHint": "Muestra combinaciones de dos ingredientes que dan el máximo número de efectos. (Respeta el filtro «Todas»/«Mejoras»/«Venenos».)",
    "triplesButton": "Combinaciones por tríos",
    "triplesHint": "Muestra combinaciones de tres ingredientes que dan el máximo número de efectos. (Respeta el filtro «Todas»/«Mejoras»/«Venenos».)"
  }
```

`pl`:
```json
  "controlPanel": {
    "searchTitle": "Szukaj kombinacji",
    "filterTypeLabel": "Typ właściwości",
    "filterAll": "Wszystkie",
    "filterBuff": "Ulepszenia",
    "filterPoison": "Trucizny",
    "findCombinationsButton": "Znajdź kombinacje",
    "ingredientCountLabel": "Składnik ({{count}})",
    "showPropertiesButton": "Pokaż właściwości",
    "pairsButton": "Kombinacje par",
    "pairsHint": "Wyświetla kombinacje dwóch składników dające maksymalną liczbę efektów. (Uwzględnia filtr „Wszystkie”/„Ulepszenia”/„Trucizny”.)",
    "triplesButton": "Kombinacje trójek",
    "triplesHint": "Wyświetla kombinacje trzech składników dające maksymalną liczbę efektów. (Uwzględnia filtr „Wszystkie”/„Ulepszenia”/„Trucizny”.)"
  }
```

`ja`:
```json
  "controlPanel": {
    "searchTitle": "組み合わせを検索",
    "filterTypeLabel": "特性の種類",
    "filterAll": "すべて",
    "filterBuff": "強化効果",
    "filterPoison": "毒",
    "findCombinationsButton": "組み合わせを検索",
    "ingredientCountLabel": "材料（{{count}}）",
    "showPropertiesButton": "特性を表示",
    "pairsButton": "ペア組み合わせ",
    "pairsHint": "最大数の効果を持つ2つの材料の組み合わせを表示します。（「すべて」/「強化効果」/「毒」フィルターに従います。）",
    "triplesButton": "トリプル組み合わせ",
    "triplesHint": "最大数の効果を持つ3つの材料の組み合わせを表示します。（「すべて」/「強化効果」/「毒」フィルターに従います。）"
  }
```

`zh-Hant`:
```json
  "controlPanel": {
    "searchTitle": "搜尋組合",
    "filterTypeLabel": "屬性類型",
    "filterAll": "全部",
    "filterBuff": "強化效果",
    "filterPoison": "毒藥",
    "findCombinationsButton": "尋找組合",
    "ingredientCountLabel": "材料（{{count}}）",
    "showPropertiesButton": "顯示屬性",
    "pairsButton": "配對組合",
    "pairsHint": "顯示能產生最多效果數的兩種材料組合。（依循「全部」/「強化效果」/「毒藥」篩選器。）",
    "triplesButton": "三重組合",
    "triplesHint": "顯示能產生最多效果數的三種材料組合。（依循「全部」/「強化效果」/「毒藥」篩選器。）"
  }
```

- [ ] **Step 2: Подключить `useTranslation` и заменить JSX в `ControlPanel.tsx`**

Импорт:
```tsx
import { useTranslation } from "react-i18next";
```

В теле компонента, после деструктуризации пропсов:
```tsx
  const { t } = useTranslation();
```

Полный `return` (значения `value` у `Radio` — `""`/`"Улучшение"`/`"Яд"` — не
менять, это `FilterKind`, см. Global Constraints):
```tsx
  return (
    <Stack gap={4} p="sm">
      <Text size="sm" fw={700}>
        {t("controlPanel.searchTitle")}
      </Text>

      {[0, 1, 2, 3].map((i) => (
        <Select
          key={i}
          label={t("common.propertyLabel", { index: i + 1 })}
          placeholder={t("common.notSelected")}
          data={propertyOptions}
          value={selects[i] !== null ? String(selects[i]) : null}
          onChange={(v) => setSelect(i, v)}
          searchable
          clearable
          comboboxProps={{ withinPortal: true }}
        />
      ))}

      <Radio.Group
        value={filter}
        onChange={(v) => onFilterChange(v as FilterKind)}
        label={t("controlPanel.filterTypeLabel")}
        mt={4}
      >
        <Stack gap={4} mt={4}>
          <Radio value="" label={t("controlPanel.filterAll")} />
          <Radio value="Улучшение" label={t("controlPanel.filterBuff")} />
          <Radio value="Яд" label={t("controlPanel.filterPoison")} />
        </Stack>
      </Radio.Group>

      <Button variant="light" mt="xs" onClick={onFindCombinations}>
        {t("controlPanel.findCombinationsButton")}
      </Button>

      <Divider my="sm" />

      <Text size="sm" fw={700}>
        {t("controlPanel.ingredientCountLabel", { count: componentNames.length })}
      </Text>
      <Select
        placeholder={t("common.notSelected")}
        data={componentOptions}
        value={componentSelect !== null ? String(componentSelect) : null}
        onChange={(v) => onComponentSelectChange(v !== null ? Number(v) : null)}
        searchable
        clearable
        comboboxProps={{ withinPortal: true }}
      />
      <Button variant="light" onClick={onShowProperties}>
        {t("controlPanel.showPropertiesButton")}
      </Button>

      <Divider my="sm" />

      <Button
        variant="light"
        onClick={onFindPairs}
        rightSection={<HintIcon label={t("controlPanel.pairsHint")} inheritColor />}
      >
        {t("controlPanel.pairsButton")}
      </Button>
      <Button
        variant="light"
        onClick={onFindMaxCombinations}
        rightSection={<HintIcon label={t("controlPanel.triplesHint")} inheritColor />}
      >
        {t("controlPanel.triplesButton")}
      </Button>
    </Stack>
  );
```

Локальную константу `EMPTY_OPTION` (строка 22) удалить — заменена
`common.notSelected`.

- [ ] **Step 3: Проверить сборку**

```bash
npx tsc -b && npm run build && npm run lint
```

- [ ] **Step 4: Commit**

```bash
git add src/components/ControlPanel.tsx src/i18n
git commit -m "feat: translate ControlPanel UI strings"
```

---

### Task 7: `ThanksModal.tsx`

**Files:**
- Modify: `src/components/ThanksModal.tsx`
- Modify: все 9 `src/i18n/locales/<lang>/translation.json`

**Interfaces:**
- Consumes: `common.close`, `common.notSelected` (Task 2).
- Produces: не потребляется другими задачами.

- [ ] **Step 1: Добавить `thanksModal.*` во все 9 файлов**

`ru`:
```json
  "thanksModal": {
    "thankYouText": "Автор благодарен Вам за поддержку!",
    "donationVariantLabel": "Вариант доната",
    "networkEthereum": "Сеть Ethereum",
    "networkSolana": "Сеть Solana",
    "networkArbitrum": "Сеть Arbitrum",
    "networkTron": "Сеть Tron"
  }
```

`en`:
```json
  "thanksModal": {
    "thankYouText": "The author thanks you for your support!",
    "donationVariantLabel": "Donation Option",
    "networkEthereum": "Ethereum Network",
    "networkSolana": "Solana Network",
    "networkArbitrum": "Arbitrum Network",
    "networkTron": "Tron Network"
  }
```

`fr`:
```json
  "thanksModal": {
    "thankYouText": "L'auteur vous remercie pour votre soutien !",
    "donationVariantLabel": "Option de don",
    "networkEthereum": "Réseau Ethereum",
    "networkSolana": "Réseau Solana",
    "networkArbitrum": "Réseau Arbitrum",
    "networkTron": "Réseau Tron"
  }
```

`de`:
```json
  "thanksModal": {
    "thankYouText": "Der Autor dankt Ihnen für Ihre Unterstützung!",
    "donationVariantLabel": "Spendenoption",
    "networkEthereum": "Ethereum-Netzwerk",
    "networkSolana": "Solana-Netzwerk",
    "networkArbitrum": "Arbitrum-Netzwerk",
    "networkTron": "Tron-Netzwerk"
  }
```

`it`:
```json
  "thanksModal": {
    "thankYouText": "L'autore ti ringrazia per il tuo supporto!",
    "donationVariantLabel": "Opzione di donazione",
    "networkEthereum": "Rete Ethereum",
    "networkSolana": "Rete Solana",
    "networkArbitrum": "Rete Arbitrum",
    "networkTron": "Rete Tron"
  }
```

`es`:
```json
  "thanksModal": {
    "thankYouText": "¡El autor te agradece tu apoyo!",
    "donationVariantLabel": "Opción de donación",
    "networkEthereum": "Red Ethereum",
    "networkSolana": "Red Solana",
    "networkArbitrum": "Red Arbitrum",
    "networkTron": "Red Tron"
  }
```

`pl`:
```json
  "thanksModal": {
    "thankYouText": "Autor dziękuje za wsparcie!",
    "donationVariantLabel": "Opcja wsparcia",
    "networkEthereum": "Sieć Ethereum",
    "networkSolana": "Sieć Solana",
    "networkArbitrum": "Sieć Arbitrum",
    "networkTron": "Sieć Tron"
  }
```

`ja`:
```json
  "thanksModal": {
    "thankYouText": "作者はあなたのご支援に感謝します！",
    "donationVariantLabel": "寄付方法",
    "networkEthereum": "Ethereumネットワーク",
    "networkSolana": "Solanaネットワーク",
    "networkArbitrum": "Arbitrumネットワーク",
    "networkTron": "Tronネットワーク"
  }
```

`zh-Hant`:
```json
  "thanksModal": {
    "thankYouText": "作者感謝您的支持！",
    "donationVariantLabel": "贊助方式",
    "networkEthereum": "Ethereum 網路",
    "networkSolana": "Solana 網路",
    "networkArbitrum": "Arbitrum 網路",
    "networkTron": "Tron 網路"
  }
```

- [ ] **Step 2: Подключить `useTranslation`, перевести `DONATION_OPTIONS` и JSX**

Импорт:
```tsx
import { useTranslation } from "react-i18next";
```

`DONATION_OPTIONS` — заменить поле `label` на `labelKey` (значение `value` —
`"ethereum"`/`"solana"`/`"arbitrum"`/`"tron"` — не менять, это внутренний id,
не отображаемый текст):
```tsx
interface DonationOption {
  value: string;
  labelKey: string;
  qrSrc: string;
  address: string;
  bannerSrc: string;
}

// Наполняется поэтапно.
const DONATION_OPTIONS: DonationOption[] = [
  {
    value: "ethereum",
    labelKey: "thanksModal.networkEthereum",
    qrSrc: ethereumQr,
    address: "0x0ce4e6492Be3C088bC13E2ba74Ffe0EE61514995",
    bannerSrc: ethereumBanner,
  },
  {
    value: "solana",
    labelKey: "thanksModal.networkSolana",
    qrSrc: solanaQr,
    address: "8rcbGs2SS4Zm9gGYucqWQmrnTx5BdDgDPg1LvbiNXJwe",
    bannerSrc: ethereumBanner,
  },
  {
    value: "arbitrum",
    labelKey: "thanksModal.networkArbitrum",
    qrSrc: arbitrumQr,
    address: "0x0ce4e6492Be3C088bC13E2ba74Ffe0EE61514995",
    bannerSrc: ethereumBanner,
  },
  {
    value: "tron",
    labelKey: "thanksModal.networkTron",
    qrSrc: tronQr,
    address: "TTRwtz3B7dBjwhUowFuqR83AGdrwRxAXuu",
    bannerSrc: ethereumBanner,
  },
];
```

Локальную константу `EMPTY_OPTION` (строка 55) удалить — заменена
`common.notSelected`.

В теле компонента, после `const selected = ...`:
```tsx
  const { t } = useTranslation();
```

JSX-замены:
```tsx
        <CloseButton
          onClick={onClose}
          aria-label={t("common.close")}
          style={{
            position: "absolute",
            top: 6,
            right: 6,
            zIndex: 1,
            background: "rgba(255,255,255,0.75)",
            borderRadius: 4,
          }}
        />
```

```tsx
          <Text size="sm" fw={600} ta="center">
            {t("thanksModal.thankYouText")}
          </Text>

          <Select
            label={t("thanksModal.donationVariantLabel")}
            placeholder={t("common.notSelected")}
            data={DONATION_OPTIONS.map((o) => ({ value: o.value, label: t(o.labelKey) }))}
            value={donation}
            onChange={setDonation}
            clearable
            comboboxProps={{ withinPortal: true }}
            w="100%"
          />
```

- [ ] **Step 3: Проверить сборку**

```bash
npx tsc -b && npm run build && npm run lint
```

- [ ] **Step 4: Commit**

```bash
git add src/components/ThanksModal.tsx src/i18n
git commit -m "feat: translate ThanksModal UI strings"
```

---

### Task 8: `AboutModal.tsx`

**Files:**
- Modify: `src/components/AboutModal.tsx`
- Modify: все 9 `src/i18n/locales/<lang>/translation.json`

**Interfaces:**
- Consumes: `common.close`, `appTitle` (Task 1 — используется повторно,
  название приложения одинаково в заголовке окна и в этом модальном окне,
  отдельный ключ был бы дублированием).
- Produces: не потребляется другими задачами.

- [ ] **Step 1: Добавить `aboutModal.*` во все 9 файлов**

`ru`:
```json
  "aboutModal": {
    "versionLabel": "Версия {{version}}",
    "authorLabel": "Автор: {{author}}",
    "summary": "Справочник ингредиентов и алхимических сочетаний Skyrim: поиск нужных эффектов и подбор компонентов по заданным свойствам.",
    "donateButton": "Автору на Эль"
  }
```

`en`:
```json
  "aboutModal": {
    "versionLabel": "Version {{version}}",
    "authorLabel": "Author: {{author}}",
    "summary": "A reference for Skyrim's alchemy ingredients and combinations: search for the effects you need and find matching ingredients by their properties.",
    "donateButton": "Donate to the Author"
  }
```

`fr`:
```json
  "aboutModal": {
    "versionLabel": "Version {{version}}",
    "authorLabel": "Auteur : {{author}}",
    "summary": "Un guide des ingrédients et combinaisons alchimiques de Skyrim : recherchez les effets voulus et trouvez les ingrédients correspondants selon leurs propriétés.",
    "donateButton": "Faire un don à l'auteur"
  }
```

`de`:
```json
  "aboutModal": {
    "versionLabel": "Version {{version}}",
    "authorLabel": "Autor: {{author}}",
    "summary": "Ein Nachschlagewerk für Skyrims Alchemie-Zutaten und -Kombinationen: Suchen Sie nach den gewünschten Effekten und finden Sie passende Zutaten anhand ihrer Eigenschaften.",
    "donateButton": "Dem Autor spenden"
  }
```

`it`:
```json
  "aboutModal": {
    "versionLabel": "Versione {{version}}",
    "authorLabel": "Autore: {{author}}",
    "summary": "Una guida agli ingredienti e alle combinazioni alchemiche di Skyrim: cerca gli effetti desiderati e trova gli ingredienti corrispondenti in base alle loro proprietà.",
    "donateButton": "Fai una donazione all'autore"
  }
```

`es`:
```json
  "aboutModal": {
    "versionLabel": "Versión {{version}}",
    "authorLabel": "Autor: {{author}}",
    "summary": "Una guía de ingredientes y combinaciones de alquimia de Skyrim: busca los efectos que necesitas y encuentra los ingredientes correspondientes según sus propiedades.",
    "donateButton": "Donar al autor"
  }
```

`pl`:
```json
  "aboutModal": {
    "versionLabel": "Wersja {{version}}",
    "authorLabel": "Autor: {{author}}",
    "summary": "Poradnik po składnikach i kombinacjach alchemicznych w Skyrim: wyszukuj potrzebne efekty i dobieraj składniki według właściwości.",
    "donateButton": "Wesprzyj autora"
  }
```

`ja`:
```json
  "aboutModal": {
    "versionLabel": "バージョン {{version}}",
    "authorLabel": "作者：{{author}}",
    "summary": "Skyrimの錬金術素材と組み合わせのリファレンス：必要な効果を検索し、特性から対応する材料を見つけられます。",
    "donateButton": "作者に寄付する"
  }
```

`zh-Hant`:
```json
  "aboutModal": {
    "versionLabel": "版本 {{version}}",
    "authorLabel": "作者：{{author}}",
    "summary": "《無界天際》煉金材料與組合參考手冊：搜尋所需效果，並依屬性找出對應材料。",
    "donateButton": "贊助作者"
  }
```

- [ ] **Step 2: Подключить `useTranslation` и заменить JSX в `AboutModal.tsx`**

Импорт:
```tsx
import { useTranslation } from "react-i18next";
```

Удалить локальную константу `SUMMARY` (строки 23-25) — заменена
`t("aboutModal.summary")`.

В теле компонента, после `const [version, setVersion] = useState...` блока:
```tsx
  const { t } = useTranslation();
```

JSX-замены:
```tsx
        <CloseButton
          onClick={onClose}
          aria-label={t("common.close")}
          style={{
            position: "absolute",
            top: 6,
            right: 6,
            zIndex: 1,
            background: "rgba(255,255,255,0.75)",
            borderRadius: 4,
          }}
        />
```

```tsx
            <Text fw={600} size="sm">
              {t("appTitle")}
            </Text>
            <Text size="xs" c="dimmed">
              {t("aboutModal.versionLabel", { version: version ?? "…" })}
            </Text>
            <Text size="xs">{t("aboutModal.authorLabel", { author: AUTHOR })}</Text>
            <Text size="xs" style={{ marginTop: 8 }}>
              {t("aboutModal.summary")}
            </Text>
```

```tsx
            <Button variant="light" size="compact-xs" onClick={onDonateClick}>
              {t("aboutModal.donateButton")}
            </Button>
```

- [ ] **Step 3: Проверить сборку**

```bash
npx tsc -b && npm run build && npm run lint
```

- [ ] **Step 4: Commit**

```bash
git add src/components/AboutModal.tsx src/i18n
git commit -m "feat: translate AboutModal UI strings"
```

---

### Task 9: `BottomPane.tsx` + `HintIcon.tsx`

Небольшая совместная задача (по аналогии с Task 2 плана hint-icons) — оба
файла меняются по одному и тому же паттерну, суммарный дифф маленький,
дробить дальше не имеет смысла.

**Files:**
- Modify: `src/components/BottomPane.tsx`
- Modify: `src/components/HintIcon.tsx`
- Modify: все 9 `src/i18n/locales/<lang>/translation.json`

**Interfaces:**
- Consumes: `common.noImage` (Task 2 — заодно исправляет непереведённую
  захардкоженную английскую строку `"No image"`, которая была в `BottomPane.tsx`
  независимо от остального интерфейса, чинившегося по-русски).
- Produces: не потребляется другими задачами.

- [ ] **Step 1: Добавить `hintIcon.*` во все 9 файлов**

`ru`: `"hintIcon": { "ariaLabel": "Подсказка" }`
`en`: `"hintIcon": { "ariaLabel": "Hint" }`
`fr`: `"hintIcon": { "ariaLabel": "Astuce" }`
`de`: `"hintIcon": { "ariaLabel": "Hinweis" }`
`it`: `"hintIcon": { "ariaLabel": "Suggerimento" }`
`es`: `"hintIcon": { "ariaLabel": "Sugerencia" }`
`pl`: `"hintIcon": { "ariaLabel": "Podpowiedź" }`
`ja`: `"hintIcon": { "ariaLabel": "ヒント" }`
`zh-Hant`: `"hintIcon": { "ariaLabel": "提示" }`

(Ключ `common.noImage` уже существует с Task 2 — новых `common.*` ключей
здесь не требуется.)

- [ ] **Step 2: `BottomPane.tsx` — заменить "No image" на `t("common.noImage")`**

Импорт:
```tsx
import { useTranslation } from "react-i18next";
```

В теле компонента, после `const imageSize = useAdaptiveImageSize(imageDataUrl);`:
```tsx
  const { t } = useTranslation();
```

Замена:
```tsx
            {mode.imageDataUrl ? (
              <Image src={mode.imageDataUrl} w={imageSize} h={imageSize} fit="contain" />
            ) : (
              <Text c="dimmed">{t("common.noImage")}</Text>
            )}
```

- [ ] **Step 3: `HintIcon.tsx` — перевести `aria-label`**

Импорт:
```tsx
import { useTranslation } from "react-i18next";
```

В теле компонента:
```tsx
export function HintIcon({ label, inheritColor = false }: HintIconProps) {
  const { t } = useTranslation();
  return (
    <Tooltip label={label} multiline w={260} withArrow>
      <ActionIcon
        variant="subtle"
        color={inheritColor ? undefined : "gray"}
        size="xs"
        onClick={(e) => e.stopPropagation()}
        aria-label={t("hintIcon.ariaLabel")}
        style={inheritColor ? { color: "inherit" } : undefined}
      >
        <IconHelpCircle size={14} />
      </ActionIcon>
    </Tooltip>
  );
}
```

- [ ] **Step 4: Проверить сборку**

```bash
npx tsc -b && npm run build && npm run lint
```

- [ ] **Step 5: Commit**

```bash
git add src/components/BottomPane.tsx src/components/HintIcon.tsx src/i18n
git commit -m "feat: translate BottomPane/HintIcon strings, fix hardcoded English noImage text"
```

---

## После выполнения плана

- [ ] **Финальная проверка ключей** — убедиться, что все 9 файлов
  `src/i18n/locales/*/translation.json` содержат ОДИНАКОВЫЙ набор ключей
  (иначе `fallbackLng: "ru"` молча маскирует пропуски). Быстрая проверка:
  ```bash
  cd src/i18n/locales
  for f in */translation.json; do node -e "console.log('$f', JSON.stringify(Object.keys(require('./$f')).sort()))"; done
  ```
  Все 9 строк должны показывать одинаковый набор ключей верхнего уровня
  (`appTitle`, `common`, `menu`, `app`, `editorModal`, `settingsModal`,
  `addons`, `controlPanel`, `thanksModal`, `aboutModal`, `hintIcon`) — при
  желании усложнить проверку до рекурсивного сравнения вложенных ключей.
- [ ] **Попросить у пользователя скриншоты** на 2-3 языках — `ru` как есть,
  один "длинный" (`de` или `pl`) и один нелатинский (`ja` или `zh-Hant`), в
  Настройках сменить язык на каждый по очереди и сверить, что кнопки/лейблы
  не переполняются и не обрезаются.
- [ ] Обновить `SESSION_NOTES.md` записью об этой сессии (файл отстаёт с
  2026-08-07, вся i18n-арка и hint-icons в него не попали — стоит
  актуализировать одним заходом, отдельная договорённость с пользователем).
