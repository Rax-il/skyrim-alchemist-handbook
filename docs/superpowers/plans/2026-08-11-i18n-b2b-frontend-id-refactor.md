# Переход фронтенда на id (План B2b) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** привести фронтенд в соответствие с уже смерженным B2a — заменить
строковое имя ингредиента/свойства на числовой `id` как основной ключ
состояния выбора во всех местах, где сейчас используется имя, и явно
передавать `lang` в каждый вызов `invoke()`, который его требует.
Отображаемое имя становится производным значением, резолвится через `id` в
момент рендера, а не хранится как сам ключ состояния.

**Architecture:** пять последовательных задач в одном фронтенд-пакете,
каждая коммитится отдельно. Task 1 — `src/lib/api.ts` (типы + формы
`invoke()`-вызовов, единый источник истины для остальных задач). Task 2/3 —
презентационные компоненты `ControlPanel.tsx`/`TopPane.tsx` (получают уже
готовые id-типизированные пропсы, сами `invoke()` не вызывают). Task 4 —
`App.tsx` (склеивает всё: состояние, обработчики, реальные вызовы `api.ts`).
Task 5 — `EditorModal.tsx` (отдельная самодостаточная модалка со своим
состоянием, тоже вызывает `api.ts`). `BottomPane.tsx` **не входит в план** —
при чтении файла подтверждено, что ни один из его пропсов не несёт
идентификатор (только уже готовые строки для отображения: `header: string`,
`results: string[]`, `BottomMode.media.name/description` — используются как
текст, не как ключ), значит менять там нечего.

**Tech Stack:** TypeScript/React/Mantine (существующий стек проекта). В
проекте нет фронтенд-тестового раннера (`package.json` — только `tsc -b`,
`vite build`, `oxlint`, ручная проверка в браузере) — как и в плане B2a
("падающий тест" в статически типизированном языке — это ошибка
компиляции), здесь RED/GREEN — это `npx tsc -b` (ошибки/чисто), а не
unit-тесты. Финальная приёмка — полностью зелёный `npm run build`; после
этого — попросить пользователя визуально проверить в браузере (см. правило
проекта "просить скриншот вместо запуска dev-сервера самостоятельно"), не
поднимать `npm run tauri dev` самостоятельно.

## Global Constraints

- **Язык статичен на весь этот план.** Реальное переключение языка — это
  B3 (отдельный план), `SettingsModal.tsx`'s language-селектор пока не
  подключён ни к чему реальному (`layout.rs` не имеет поля `language`).
  Здесь `lang`, передаваемый в каждый вызов `invoke()`, который его
  требует — это константа `CURRENT_LANG = "ru"`, экспортируемая из
  `api.ts` (заменяет 9 разбросанных литералов `"ru"` одним местом для
  будущей замены в B3). Никакого нового UI/state для языка в этом плане не
  добавляется.
- **Формы `invoke()` должны совпадать 1-в-1 с `src-tauri/src/commands.rs`**
  (уже смержен, финальные сигнатуры проверены чтением файла напрямую, не
  из спеки — спека писалась до того, как B2a был реализован в деталях):
  - `get_properties(lang)` → `Vec<PropertyInfo>` (`{id, name}`)
  - `get_component_names(lang)` → `Vec<ComponentNameInfo>` (`{id, name}`)
  - `get_component_names_filtered(addons, lang)` → `Vec<ComponentNameInfo>`
  - `get_component_properties(id, lang)` → `Vec<String>` (переведённые
    имена свойств компонента — **остаётся строками**, см. ниже)
  - `get_component_properties_with_types(id, lang)` → `Vec<PropWithType>`
    (`{id, name, typ}` — раньше было только `{name, typ}`)
  - `get_component_media(id, lang)` → `ComponentMedia` (без изменений формы)
  - `find_combinations(selected: number[], filter, addons, lang)` →
    `Vec<CombinationResult>`, где `CombinationResult.components` теперь
    `number[]` (было `string[]`)
  - `find_pairs(filter, addons, maxResults, lang)` → `string[]` (без
    изменений — как и раньше, уже готовый текстовый вывод построчно)
  - `find_max_combinations(filter, addons, maxResults, lang)` → `string[]`
  - `component_exists(name, lang)` → `boolean` (имя, не id — новой записи
    id ещё не существует; проверка теперь в рамках конкретного языка)
  - `is_user_added_component(id)` → `boolean` (без `lang`)
  - `insert_component(name, lang, props: number[4])` → `number` (id новой
    записи; имя, не id — по той же причине, что и `component_exists`)
  - `delete_component(id)` → `void` (без `lang`)
  - `update_component_properties(id, props: number[4])` → `void` (без
    `lang`)
  - `set_component_media(id, lang, imageBase64, description)` → `void`
  - `rename_component` — **удалена целиком с Rust-стороны** (B2a), в
    `api.ts` тоже удаляется полностью — не вызывалась из UI уже до этого
    плана (см. `SESSION_NOTES.md`, инцидент с "Аронией"), удаление не несёт
    поведенческого риска.
  - Остальные команды (`pick_image_file`, `get_layout`, `save_layout`,
    `save_scale`, `save_addons`, `save_max_combinations`) не меняются в
    этом плане — их сигнатуры не зависят от id/lang.
- **`TopMode.checklist`** (`TopPane.tsx`) хранит `ComponentNameInfo[]`
  (`{id, name}`), не `string[]` — переключение `enabledComponents` теперь
  по `id: number`, не по имени. **`TopMode.properties`** (список свойств
  выбранного компонента, только отображение, дальше никуда не передаётся
  как идентификатор) **остаётся `string[]`** — сознательное решение из
  спеки, не менять.
- **`enabledComponents`** везде (`App.tsx`/`TopPane.tsx`) —
  `Record<number, boolean>` (id → boolean), было `Record<string, boolean>`.
- **`EditorModal.tsx`'s `propSelects`** — `[number | null, number | null,
  number | null, number | null]` (id свойств), было `[string, string,
  string, string]` (имена). Валидация на пустоту/дубликаты — по `null`/
  `Set<number>` вместо `""`/`Set<string>`.
- **`EditorModal.tsx`'s состояние выбранного ингредиента** разделяется на
  два поля: `loadedId: number | null` (идентификатор для всех
  id-based вызовов; `null`, пока не загружено ничего или пока это ещё не
  сохранённый новый компонент) и `loadedName: string` (отображаемое имя —
  для `TextInput` в режиме "новый" и текста диалога удаления). Раньше одно
  поле `loadedName: string` совмещало обе роли.

---

### Task 1: `src/lib/api.ts`

**Files:**
- Modify: `src/lib/api.ts`

**Interfaces:**
- Consumes: `src-tauri/src/commands.rs` (уже смержен, не меняется этим
  планом) — единственный источник истины для форм `invoke()`.
- Produces (используется во всех остальных задачах):
  - `export interface PropertyInfo { id: number; name: string }`
  - `export interface ComponentNameInfo { id: number; name: string }`
  - `export interface PropWithType { id: number; name: string; typ: string }`
  - `export interface CombinationResult { components: number[]; line: string }`
  - `export const CURRENT_LANG = "ru"`
  - Обновлённый объект `api` (сигнатуры — см. Global Constraints выше).

- [ ] **Step 1: Переписать `api.ts` целиком**

Замените содержимое `src/lib/api.ts` (с той же вводной шапкой-комментарием)
на:

```typescript
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
```

Ключевые отличия от старой версии: все читающие функции, которым нужен
язык, принимают `lang: string`; `renameComponent`/`rename_component`
удалены целиком; `insertComponent`/`updateComponentProperties`/
`setComponentMedia`/`deleteComponent` — на `id`/`props: number[4]`;
`componentExists`/`insertComponent` остаются на имени (плюс `lang`), как и
задокументировано в Global Constraints.

- [ ] **Step 2: Проверить, что ошибки компиляции теперь только в файлах, ещё не тронутых этим планом**

Run: `npx tsc -b`
Expected: FAIL — ошибки в `App.tsx`, `ControlPanel.tsx`, `TopPane.tsx`,
`EditorModal.tsx` (используют старые формы `api.*`/старые типы). Файлов
`BottomPane.tsx`/`addons.ts`/остальных модулей `lib/` в списке ошибок быть
не должно — `api.ts` сам по себе валиден.

- [ ] **Step 3: Commit**

```bash
git add src/lib/api.ts
git commit -m "feat: switch api.ts to id-based commands + lang param (Task 1 of B2b)"
```

---

### Task 2: `src/components/ControlPanel.tsx`

**Files:**
- Modify: `src/components/ControlPanel.tsx`

**Interfaces:**
- Consumes: `PropertyInfo`, `ComponentNameInfo` из `../lib/api` (Task 1).
- Produces (используется в Task 4, `App.tsx`):
  - `Props.properties: PropertyInfo[]`
  - `Props.selects: [number | null, number | null, number | null, number | null]`
  - `Props.onSelectsChange: (next: [number | null, number | null, number | null, number | null]) => void`
  - `Props.componentNames: ComponentNameInfo[]`
  - `Props.componentSelect: number | null`
  - `Props.onComponentSelectChange: (v: number | null) => void`
  - Остальные поля `Props` (`filter`, `onFilterChange`, `onFindCombinations`,
    `onShowProperties`, `onFindPairs`, `onFindMaxCombinations`) без
    изменений.

- [ ] **Step 1: Обновить `Props` и внутреннюю логику**

Замените:

```typescript
import { Button, Divider, Radio, Select, Stack, Text } from "@mantine/core";
import type { FilterKind } from "../lib/api";

interface Props {
  properties: string[];
  selects: [string, string, string, string];
  onSelectsChange: (next: [string, string, string, string]) => void;
  filter: FilterKind;
  onFilterChange: (f: FilterKind) => void;
  componentNames: string[];
  componentSelect: string;
  onComponentSelectChange: (v: string) => void;
  onFindCombinations: () => void;
  onShowProperties: () => void;
  onFindPairs: () => void;
  onFindMaxCombinations: () => void;
}

const EMPTY_OPTION = "— не выбрано —";

export function ControlPanel({
  properties,
  selects,
  onSelectsChange,
  filter,
  onFilterChange,
  componentNames,
  componentSelect,
  onComponentSelectChange,
  onFindCombinations,
  onShowProperties,
  onFindPairs,
  onFindMaxCombinations,
}: Props) {
  const propertyOptions = properties.map((p) => ({ value: p, label: p }));
  const componentOptions = componentNames.map((n) => ({ value: n, label: n }));

  const setSelect = (i: number, value: string | null) => {
    const next = [...selects] as [string, string, string, string];
    next[i] = value ?? "";
    onSelectsChange(next);
  };
```

на:

```typescript
import { Button, Divider, Radio, Select, Stack, Text } from "@mantine/core";
import type { ComponentNameInfo, FilterKind, PropertyInfo } from "../lib/api";

type Selects = [number | null, number | null, number | null, number | null];

interface Props {
  properties: PropertyInfo[];
  selects: Selects;
  onSelectsChange: (next: Selects) => void;
  filter: FilterKind;
  onFilterChange: (f: FilterKind) => void;
  componentNames: ComponentNameInfo[];
  componentSelect: number | null;
  onComponentSelectChange: (v: number | null) => void;
  onFindCombinations: () => void;
  onShowProperties: () => void;
  onFindPairs: () => void;
  onFindMaxCombinations: () => void;
}

const EMPTY_OPTION = "— не выбрано —";

export function ControlPanel({
  properties,
  selects,
  onSelectsChange,
  filter,
  onFilterChange,
  componentNames,
  componentSelect,
  onComponentSelectChange,
  onFindCombinations,
  onShowProperties,
  onFindPairs,
  onFindMaxCombinations,
}: Props) {
  const propertyOptions = properties.map((p) => ({ value: String(p.id), label: p.name }));
  const componentOptions = componentNames.map((c) => ({ value: String(c.id), label: c.name }));

  const setSelect = (i: number, value: string | null) => {
    const next = [...selects] as Selects;
    next[i] = value !== null ? Number(value) : null;
    onSelectsChange(next);
  };
```

Дальше в JSX замените два места, где значение Select'а строится из
"сырого" state:

```typescript
          value={selects[i] || null}
```

на:

```typescript
          value={selects[i] !== null ? String(selects[i]) : null}
```

и:

```typescript
        value={componentSelect || null}
        onChange={(v) => onComponentSelectChange(v ?? "")}
```

на:

```typescript
        value={componentSelect !== null ? String(componentSelect) : null}
        onChange={(v) => onComponentSelectChange(v !== null ? Number(v) : null)}
```

- [ ] **Step 2: Проверить компиляцию**

Run: `npx tsc -b`
Expected: FAIL — ошибки теперь в `App.tsx` (передаёт `ControlPanel` пропсы
старой формы) и всё ещё в `TopPane.tsx`/`EditorModal.tsx`. Ошибок внутри
самого `ControlPanel.tsx` быть не должно.

- [ ] **Step 3: Commit**

```bash
git add src/components/ControlPanel.tsx
git commit -m "feat: switch ControlPanel.tsx props to id-based selects (Task 2 of B2b)"
```

---

### Task 3: `src/components/TopPane.tsx`

**Files:**
- Modify: `src/components/TopPane.tsx`

**Interfaces:**
- Consumes: `ComponentNameInfo` из `../lib/api` (Task 1).
- Produces (используется в Task 4, `App.tsx`):
  - `export type TopMode = { kind: "empty" } | { kind: "checklist"; items: ComponentNameInfo[] } | { kind: "properties"; props: string[] }`
  - `Props.enabledComponents: Record<number, boolean>`
  - `Props.onToggleComponent: (id: number, checked: boolean) => void`

- [ ] **Step 1: Обновить типы и рендер checklist-режима**

Замените весь файл:

```typescript
import { Checkbox, ScrollArea, Stack, Text } from "@mantine/core";

export type TopMode =
  | { kind: "empty" }
  | { kind: "checklist"; names: string[] }
  | { kind: "properties"; props: string[] };

interface Props {
  mode: TopMode;
  enabledComponents: Record<string, boolean>;
  onToggleComponent: (name: string, checked: boolean) => void;
}

export function TopPane({ mode, enabledComponents, onToggleComponent }: Props) {
  if (mode.kind === "empty") {
    return (
      <Text size="sm" c="dimmed" p="xs">
        Список ингредиентов (0)
      </Text>
    );
  }

  if (mode.kind === "checklist") {
    return (
      <Stack gap={0} h="100%">
        <Text size="sm" fw={700} p="xs" pb={4}>
          Список ингредиентов ({mode.names.length})
        </Text>
        <ScrollArea flex={1} px="xs">
          <Stack gap={2}>
            {mode.names.map((name) => (
              <Checkbox
                key={name}
                label={name}
                checked={enabledComponents[name] ?? true}
                onChange={(e) => onToggleComponent(name, e.currentTarget.checked)}
              />
            ))}
          </Stack>
        </ScrollArea>
      </Stack>
    );
  }

  return (
    <Stack gap={0} h="100%">
      <Text size="sm" fw={700} p="xs" pb={4}>
        Свойства компонента ({mode.props.length})
      </Text>
      <ScrollArea flex={1} px="xs">
        <Stack gap={2}>
          {mode.props.map((p) => (
            <Text key={p} size="sm">
              {p}
            </Text>
          ))}
        </Stack>
      </ScrollArea>
    </Stack>
  );
}
```

на:

```typescript
import { Checkbox, ScrollArea, Stack, Text } from "@mantine/core";
import type { ComponentNameInfo } from "../lib/api";

export type TopMode =
  | { kind: "empty" }
  | { kind: "checklist"; items: ComponentNameInfo[] }
  | { kind: "properties"; props: string[] };

interface Props {
  mode: TopMode;
  enabledComponents: Record<number, boolean>;
  onToggleComponent: (id: number, checked: boolean) => void;
}

export function TopPane({ mode, enabledComponents, onToggleComponent }: Props) {
  if (mode.kind === "empty") {
    return (
      <Text size="sm" c="dimmed" p="xs">
        Список ингредиентов (0)
      </Text>
    );
  }

  if (mode.kind === "checklist") {
    return (
      <Stack gap={0} h="100%">
        <Text size="sm" fw={700} p="xs" pb={4}>
          Список ингредиентов ({mode.items.length})
        </Text>
        <ScrollArea flex={1} px="xs">
          <Stack gap={2}>
            {mode.items.map((c) => (
              <Checkbox
                key={c.id}
                label={c.name}
                checked={enabledComponents[c.id] ?? true}
                onChange={(e) => onToggleComponent(c.id, e.currentTarget.checked)}
              />
            ))}
          </Stack>
        </ScrollArea>
      </Stack>
    );
  }

  return (
    <Stack gap={0} h="100%">
      <Text size="sm" fw={700} p="xs" pb={4}>
        Свойства компонента ({mode.props.length})
      </Text>
      <ScrollArea flex={1} px="xs">
        <Stack gap={2}>
          {mode.props.map((p) => (
            <Text key={p} size="sm">
              {p}
            </Text>
          ))}
        </Stack>
      </ScrollArea>
    </Stack>
  );
}
```

(Только `checklist`-ветка меняется по смыслу; `properties`-ветка
переписана дословно той же — включена в замену только потому, что стоит
внутри той же функции.)

- [ ] **Step 2: Проверить компиляцию**

Run: `npx tsc -b`
Expected: FAIL — ошибки теперь только в `App.tsx` (передаёт старые формы
`TopMode`/`enabledComponents`) и `EditorModal.tsx`. Ошибок внутри
`TopPane.tsx` быть не должно.

- [ ] **Step 3: Commit**

```bash
git add src/components/TopPane.tsx
git commit -m "feat: switch TopPane.tsx checklist mode to id-based items (Task 3 of B2b)"
```

---

### Task 4: `src/App.tsx`

**Files:**
- Modify: `src/App.tsx`

**Interfaces:**
- Consumes: `api`, `CURRENT_LANG`, `PropertyInfo`, `ComponentNameInfo`,
  `CombinationResult` из `./lib/api` (Task 1); `ControlPanel`'s новый
  `Props` (Task 2); `TopMode`/`TopPane`'s новый `Props` (Task 3).
- Produces: ничего, что потребляют более поздние задачи — `EditorModal`
  (Task 5) получает список компонентов независимо через свой собственный
  `api.getComponentNames()`, не через `App.tsx`.

- [ ] **Step 1: Обновить типы состояния**

Замените:

```typescript
  const [properties, setProperties] = useState<string[]>([]);
  const [componentNames, setComponentNames] = useState<string[]>([]);

  const [selects, setSelects] = useState<[string, string, string, string]>(["", "", "", ""]);
  const [filter, setFilter] = useState<FilterKind>("");
  const [componentSelect, setComponentSelect] = useState("");
```

на:

```typescript
  const [properties, setProperties] = useState<PropertyInfo[]>([]);
  const [componentNames, setComponentNames] = useState<ComponentNameInfo[]>([]);

  const [selects, setSelects] = useState<[number | null, number | null, number | null, number | null]>([
    null,
    null,
    null,
    null,
  ]);
  const [filter, setFilter] = useState<FilterKind>("");
  const [componentSelect, setComponentSelect] = useState<number | null>(null);
```

и:

```typescript
  const lastCombosRef = useRef<CombinationResult[]>([]);
  const [enabledComponents, setEnabledComponents] = useState<Record<string, boolean>>({});
```

на:

```typescript
  const lastCombosRef = useRef<CombinationResult[]>([]);
  const [enabledComponents, setEnabledComponents] = useState<Record<number, boolean>>({});
```

Добавьте импорт `PropertyInfo`, `ComponentNameInfo` рядом с уже
существующим импортом типов из `./lib/api`:

```typescript
import { api } from "./lib/api";
import type { CombinationResult, FilterKind } from "./lib/api";
```

на:

```typescript
import { api, CURRENT_LANG } from "./lib/api";
import type { ComponentNameInfo, CombinationResult, FilterKind, PropertyInfo } from "./lib/api";
```

- [ ] **Step 2: Обновить загрузку данных**

Замените:

```typescript
  useEffect(() => {
    api.getProperties().then(setProperties);
    api.getLayout().then((l) => {
      setSidePanelWidth(l.side_panel_width);
      setSplitRatio(l.split_ratio);
      setEnabledAddons(l.enabled_addons);
      setMaxCombinations(l.max_combinations);
    });
  }, []);
```

на:

```typescript
  useEffect(() => {
    api.getProperties(CURRENT_LANG).then(setProperties);
    api.getLayout().then((l) => {
      setSidePanelWidth(l.side_panel_width);
      setSplitRatio(l.split_ratio);
      setEnabledAddons(l.enabled_addons);
      setMaxCombinations(l.max_combinations);
    });
  }, []);
```

Замените:

```typescript
  useEffect(() => {
    api.getComponentNamesFiltered(enabledAddons).then((names) => {
      setComponentNames(names);
      setComponentSelect((prev) => (names.includes(prev) ? prev : ""));
    });
    lastCombosRef.current = [];
    setEnabledComponents({});
    setTopMode({ kind: "empty" });
    setBottomMode({ kind: "list" });
    setResults([]);
    setResultsHeader("Найдено 0 комбинаций");
  }, [enabledAddons]);

  function refreshLists() {
    api.getProperties().then(setProperties);
    api.getComponentNamesFiltered(enabledAddons).then(setComponentNames);
  }
```

на:

```typescript
  useEffect(() => {
    api.getComponentNamesFiltered(enabledAddons, CURRENT_LANG).then((names) => {
      setComponentNames(names);
      setComponentSelect((prev) => (prev !== null && names.some((n) => n.id === prev) ? prev : null));
    });
    lastCombosRef.current = [];
    setEnabledComponents({});
    setTopMode({ kind: "empty" });
    setBottomMode({ kind: "list" });
    setResults([]);
    setResultsHeader("Найдено 0 комбинаций");
  }, [enabledAddons]);

  function refreshLists() {
    api.getProperties(CURRENT_LANG).then(setProperties);
    api.getComponentNamesFiltered(enabledAddons, CURRENT_LANG).then(setComponentNames);
  }
```

- [ ] **Step 3: Обновить обработчики поиска**

Замените:

```typescript
  function applyComponentFilter(enabled: Record<string, boolean>) {
    const filtered = lastCombosRef.current
      .filter((c) => c.components.every((name) => enabled[name] ?? false))
      .map((c) => c.line);
    setPlainResults(filtered, "Сочетаний не найдено");
  }

  async function handleFindCombinations() {
    const chosen = selects.filter((s) => s !== "");
    if (chosen.length === 0) {
      info_("Внимание", "Выберите хотя бы одно свойство.");
      return;
    }
    try {
      const found = await api.findCombinations(chosen, filter, enabledAddons);
      lastCombosRef.current = found;
      const enabled: Record<string, boolean> = {};
      const names: string[] = [];
      for (const c of found) {
        for (const name of c.components) {
          if (!(name in enabled)) {
            enabled[name] = true;
            names.push(name);
          }
        }
      }
      names.sort((a, b) => a.localeCompare(b));
      setEnabledComponents(enabled);
      setTopMode({ kind: "checklist", names });
      applyComponentFilter(enabled);
    } catch (e) {
      info_("Ошибка", String(e));
    }
  }

  async function handleShowProperties() {
    if (!componentSelect) {
      info_("Внимание", "Выберите компонент.");
      return;
    }
    try {
      const props = await api.getComponentProperties(componentSelect);
      lastCombosRef.current = [];
      setEnabledComponents({});
      setTopMode({ kind: "properties", props });
      setResultsHeader(componentSelect);

      const media = await api.getComponentMedia(componentSelect);
      setBottomMode({
        kind: "media",
        name: componentSelect,
        description: media.description,
        imageDataUrl: media.image_data_url,
      });
    } catch (e) {
      info_("Ошибка", String(e));
    }
  }

  async function handleFindPairs() {
    try {
      const found = await api.findPairs(filter, enabledAddons, maxCombinations);
      lastCombosRef.current = [];
      setEnabledComponents({});
      setTopMode({ kind: "checklist", names: [] });
      setBlockResults(found, "Сочетаний не найдено");
    } catch (e) {
      info_("Ошибка", String(e));
    }
  }

  async function handleFindMaxCombinations() {
    try {
      const found = await api.findMaxCombinations(filter, enabledAddons, maxCombinations);
      lastCombosRef.current = [];
      setEnabledComponents({});
      setTopMode({ kind: "checklist", names: [] });
      setBlockResults(found, "Сочетаний не найдено");
    } catch (e) {
      info_("Ошибка", String(e));
    }
  }

  function handleToggleComponent(name: string, checked: boolean) {
    const next = { ...enabledComponents, [name]: checked };
    setEnabledComponents(next);
    applyComponentFilter(next);
  }
```

на:

```typescript
  function applyComponentFilter(enabled: Record<number, boolean>) {
    const filtered = lastCombosRef.current
      .filter((c) => c.components.every((id) => enabled[id] ?? false))
      .map((c) => c.line);
    setPlainResults(filtered, "Сочетаний не найдено");
  }

  async function handleFindCombinations() {
    const chosen = selects.filter((s): s is number => s !== null);
    if (chosen.length === 0) {
      info_("Внимание", "Выберите хотя бы одно свойство.");
      return;
    }
    try {
      const found = await api.findCombinations(chosen, filter, enabledAddons, CURRENT_LANG);
      lastCombosRef.current = found;
      const nameById = new Map(componentNames.map((c) => [c.id, c.name]));
      const enabled: Record<number, boolean> = {};
      const items: ComponentNameInfo[] = [];
      for (const c of found) {
        for (const id of c.components) {
          if (!(id in enabled)) {
            enabled[id] = true;
            items.push({ id, name: nameById.get(id) ?? String(id) });
          }
        }
      }
      items.sort((a, b) => a.name.localeCompare(b.name));
      setEnabledComponents(enabled);
      setTopMode({ kind: "checklist", items });
      applyComponentFilter(enabled);
    } catch (e) {
      info_("Ошибка", String(e));
    }
  }

  async function handleShowProperties() {
    if (componentSelect === null) {
      info_("Внимание", "Выберите компонент.");
      return;
    }
    try {
      const props = await api.getComponentProperties(componentSelect, CURRENT_LANG);
      const selectedName = componentNames.find((c) => c.id === componentSelect)?.name ?? "";
      lastCombosRef.current = [];
      setEnabledComponents({});
      setTopMode({ kind: "properties", props });
      setResultsHeader(selectedName);

      const media = await api.getComponentMedia(componentSelect, CURRENT_LANG);
      setBottomMode({
        kind: "media",
        name: selectedName,
        description: media.description,
        imageDataUrl: media.image_data_url,
      });
    } catch (e) {
      info_("Ошибка", String(e));
    }
  }

  async function handleFindPairs() {
    try {
      const found = await api.findPairs(filter, enabledAddons, maxCombinations, CURRENT_LANG);
      lastCombosRef.current = [];
      setEnabledComponents({});
      setTopMode({ kind: "checklist", items: [] });
      setBlockResults(found, "Сочетаний не найдено");
    } catch (e) {
      info_("Ошибка", String(e));
    }
  }

  async function handleFindMaxCombinations() {
    try {
      const found = await api.findMaxCombinations(filter, enabledAddons, maxCombinations, CURRENT_LANG);
      lastCombosRef.current = [];
      setEnabledComponents({});
      setTopMode({ kind: "checklist", items: [] });
      setBlockResults(found, "Сочетаний не найдено");
    } catch (e) {
      info_("Ошибка", String(e));
    }
  }

  function handleToggleComponent(id: number, checked: boolean) {
    const next = { ...enabledComponents, [id]: checked };
    setEnabledComponents(next);
    applyComponentFilter(next);
  }
```

- [ ] **Step 4: Проверить компиляцию**

Run: `npx tsc -b`
Expected: FAIL — ошибки только внутри `EditorModal.tsx` (ещё не тронут).
`App.tsx`/`ControlPanel.tsx`/`TopPane.tsx` должны компилироваться чисто
относительно друг друга.

- [ ] **Step 5: Commit**

```bash
git add src/App.tsx
git commit -m "feat: switch App.tsx search/results state to id-based (Task 4 of B2b)"
```

---

### Task 5: `src/components/EditorModal.tsx`

**Files:**
- Modify: `src/components/EditorModal.tsx`

**Interfaces:**
- Consumes: `api`, `CURRENT_LANG`, `PropertyInfo`, `ComponentNameInfo`,
  `PropWithType` из `../lib/api` (Task 1).
- Produces: ничего, что потребляют другие задачи этого плана — последняя
  задача, после неё весь проект должен собираться.

- [ ] **Step 1: Обновить импорты, типы состояния и `Snapshot`**

Замените:

```typescript
import { api } from "../lib/api";
import { GOOD_QUALITY_SIZE, useAdaptiveImageSize } from "../lib/useAdaptiveImageSize";

interface Props {
  opened: boolean;
  onClose: () => void;
  onChanged: () => void; // список компонентов на главном экране мог измениться
}

interface Snapshot {
  props: [string, string, string, string];
  imageBase64: string | null;
}

const emptySnapshot: Snapshot = { props: ["", "", "", ""], imageBase64: null };

const PREVIEW_BOX = GOOD_QUALITY_SIZE;
const DESCRIPTION_HEIGHT = 85;

export function EditorModal({ opened, onClose, onChanged }: Props) {
  const [names, setNames] = useState<string[]>([]);
  const [allProperties, setAllProperties] = useState<string[]>([]);

  const [loadedName, setLoadedName] = useState("");
  const [isNew, setIsNew] = useState(false);
  // Название/свойства/удаление разрешены только для ингредиентов, которые
  // сам пользователь добавил через "Новый" (Addon::UserAdded) — у
  // остальных нет безопасного пути восстановления при ошибке.
  const [editable, setEditable] = useState(true);

  const [newDialogOpen, setNewDialogOpen] = useState(false);
  const [newDialogGeneration, setNewDialogGeneration] = useState(0);
  const newNameRef = useRef<HTMLInputElement>(null);

  const [propSelects, setPropSelects] = useState<[string, string, string, string]>(["", "", "", ""]);
```

на:

```typescript
import { api, CURRENT_LANG } from "../lib/api";
import type { ComponentNameInfo, PropertyInfo } from "../lib/api";
import { GOOD_QUALITY_SIZE, useAdaptiveImageSize } from "../lib/useAdaptiveImageSize";

interface Props {
  opened: boolean;
  onClose: () => void;
  onChanged: () => void; // список компонентов на главном экране мог измениться
}

type PropIds = [number | null, number | null, number | null, number | null];

interface Snapshot {
  props: PropIds;
  imageBase64: string | null;
}

const emptySnapshot: Snapshot = { props: [null, null, null, null], imageBase64: null };

const PREVIEW_BOX = GOOD_QUALITY_SIZE;
const DESCRIPTION_HEIGHT = 85;

export function EditorModal({ opened, onClose, onChanged }: Props) {
  const [names, setNames] = useState<ComponentNameInfo[]>([]);
  const [allProperties, setAllProperties] = useState<PropertyInfo[]>([]);

  // loadedId — идентификатор для всех id-based вызовов; null, пока не
  // загружено ничего или пока это ещё не сохранённый новый компонент (у
  // него id появится только после insertComponent). loadedName —
  // отображаемое имя: для существующего компонента резолвится из names,
  // для нового — то, что ввёл пользователь в диалоге "Новый".
  const [loadedId, setLoadedId] = useState<number | null>(null);
  const [loadedName, setLoadedName] = useState("");
  const [isNew, setIsNew] = useState(false);
  // Название/свойства/удаление разрешены только для ингредиентов, которые
  // сам пользователь добавил через "Новый" (Addon::UserAdded) — у
  // остальных нет безопасного пути восстановления при ошибке.
  const [editable, setEditable] = useState(true);

  const [newDialogOpen, setNewDialogOpen] = useState(false);
  const [newDialogGeneration, setNewDialogGeneration] = useState(0);
  const newNameRef = useRef<HTMLInputElement>(null);

  const [propSelects, setPropSelects] = useState<PropIds>([null, null, null, null]);
```

- [ ] **Step 2: Обновить `pendingAction`, загрузку списков, `clearFields`/`loadComponent`/`startNew`**

Замените:

```typescript
  const [pendingAction, setPendingAction] = useState<
    { kind: "load"; name: string } | { kind: "close" } | { kind: "new" } | null
  >(null);
```

на:

```typescript
  const [pendingAction, setPendingAction] = useState<
    { kind: "load"; id: number; name: string } | { kind: "close" } | { kind: "new" } | null
  >(null);
```

Замените:

```typescript
  useEffect(() => {
    if (!opened) return;
    api.getProperties().then(setAllProperties).catch(() => {});
    api
      .getComponentNames()
      .then((ns) => {
        setNames(ns);
        if (ns.length > 0) loadComponent(ns[0]);
        else clearFields();
      })
      .catch(() => {});
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [opened]);
```

на:

```typescript
  useEffect(() => {
    if (!opened) return;
    api.getProperties(CURRENT_LANG).then(setAllProperties).catch(() => {});
    api
      .getComponentNames(CURRENT_LANG)
      .then((ns) => {
        setNames(ns);
        if (ns.length > 0) loadComponent(ns[0].id, ns[0].name);
        else clearFields();
      })
      .catch(() => {});
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [opened]);
```

Замените:

```typescript
  function clearFields() {
    setImageBase64(null);
    setImageFileName("");
    setDescription("");
    setDescriptionTouched(false);
    setPropSelects(["", "", "", ""]);
    setLoadedName("");
    setIsNew(false);
    setEditable(true);
    originalRef.current = emptySnapshot;
    setDirty(false);
  }

  async function loadComponent(name: string) {
    const [props, media, userAdded] = await Promise.all([
      api.getComponentPropertiesWithTypes(name),
      api.getComponentMedia(name),
      api.isUserAddedComponent(name),
    ]);
    const propNames = props.slice(0, 4).map((p) => p.name);
    while (propNames.length < 4) propNames.push("");
    const propsTuple = propNames as [string, string, string, string];
    const b64 = media.image_data_url ? base64FromDataUrl(media.image_data_url) : null;

    setPropSelects(propsTuple);
    setImageBase64(b64);
    setImageFileName("");
    setDescription(media.description);
    setDescriptionTouched(false);

    setLoadedName(name);
    setIsNew(false);
    setEditable(userAdded);
    originalRef.current = { props: propsTuple, imageBase64: b64 };
    setDirty(false);
  }

  function startNew(name: string) {
    setImageBase64(null);
    setImageFileName("");
    setDescription("");
    setDescriptionTouched(false);
    setPropSelects(["", "", "", ""]);
    setLoadedName(name);
    setIsNew(true);
    setEditable(true);
    originalRef.current = emptySnapshot;
    setDirty(false);
  }

  function requestAction(action: { kind: "load"; name: string } | { kind: "close" } | { kind: "new" }) {
    if (!dirty) {
      runAction(action);
    } else {
      setPendingAction(action);
    }
  }

  function runAction(action: { kind: "load"; name: string } | { kind: "close" } | { kind: "new" }) {
    if (action.kind === "load") loadComponent(action.name);
    else if (action.kind === "new") {
      setNewDialogGeneration((g) => g + 1);
      setNewDialogOpen(true);
    } else onClose();
  }

  async function confirmNewName() {
    const name = (newNameRef.current?.value ?? "").trim();
    if (!name) {
      setInfo({ title: "Ошибка", text: "Введите название ингредиента." });
      return;
    }
    if (await api.componentExists(name)) {
      setInfo({ title: "Ошибка", text: `Ингредиент «${name}» уже существует.` });
      return;
    }
    setNewDialogOpen(false);
    startNew(name);
  }
```

на:

```typescript
  function clearFields() {
    setImageBase64(null);
    setImageFileName("");
    setDescription("");
    setDescriptionTouched(false);
    setPropSelects([null, null, null, null]);
    setLoadedId(null);
    setLoadedName("");
    setIsNew(false);
    setEditable(true);
    originalRef.current = emptySnapshot;
    setDirty(false);
  }

  async function loadComponent(id: number, name: string) {
    const [props, media, userAdded] = await Promise.all([
      api.getComponentPropertiesWithTypes(id, CURRENT_LANG),
      api.getComponentMedia(id, CURRENT_LANG),
      api.isUserAddedComponent(id),
    ]);
    const propIds: PropIds = [null, null, null, null];
    props.slice(0, 4).forEach((p, i) => {
      propIds[i] = p.id;
    });
    const b64 = media.image_data_url ? base64FromDataUrl(media.image_data_url) : null;

    setPropSelects(propIds);
    setImageBase64(b64);
    setImageFileName("");
    setDescription(media.description);
    setDescriptionTouched(false);

    setLoadedId(id);
    setLoadedName(name);
    setIsNew(false);
    setEditable(userAdded);
    originalRef.current = { props: propIds, imageBase64: b64 };
    setDirty(false);
  }

  function startNew(name: string) {
    setImageBase64(null);
    setImageFileName("");
    setDescription("");
    setDescriptionTouched(false);
    setPropSelects([null, null, null, null]);
    setLoadedId(null);
    setLoadedName(name);
    setIsNew(true);
    setEditable(true);
    originalRef.current = emptySnapshot;
    setDirty(false);
  }

  function requestAction(
    action: { kind: "load"; id: number; name: string } | { kind: "close" } | { kind: "new" },
  ) {
    if (!dirty) {
      runAction(action);
    } else {
      setPendingAction(action);
    }
  }

  function runAction(action: { kind: "load"; id: number; name: string } | { kind: "close" } | { kind: "new" }) {
    if (action.kind === "load") loadComponent(action.id, action.name);
    else if (action.kind === "new") {
      setNewDialogGeneration((g) => g + 1);
      setNewDialogOpen(true);
    } else onClose();
  }

  async function confirmNewName() {
    const name = (newNameRef.current?.value ?? "").trim();
    if (!name) {
      setInfo({ title: "Ошибка", text: "Введите название ингредиента." });
      return;
    }
    if (await api.componentExists(name, CURRENT_LANG)) {
      setInfo({ title: "Ошибка", text: `Ингредиент «${name}» уже существует.` });
      return;
    }
    setNewDialogOpen(false);
    startNew(name);
  }
```

- [ ] **Step 3: Обновить валидацию, `performSave`, `handleDelete`**

Замените:

```typescript
  async function handleSave() {
    const seen = new Set<string>();
    for (let i = 0; i < 4; i++) {
      const p = propSelects[i];
      if (!p) {
        setInfo({ title: "Ошибка", text: `Выберите значение во всех 4 полях «Свойство» (поле ${i + 1}).` });
        return;
      }
      if (seen.has(p)) {
        setInfo({ title: "Ошибка", text: "Свойства компонента не должны повторяться." });
        return;
      }
      seen.add(p);
    }

    // Для не-пользовательских ингредиентов свойства/название всё равно
    // заблокированы в форме — тут реально может меняться только картинка
    // и описание, но это тоже правка "базовых" данных, поэтому отдельное
    // подтверждение.
    if (!isNew && !editable) {
      setConfirmBaseEdit(true);
      return;
    }
    await performSave();
  }

  async function performSave() {
    const descriptionValue = (descriptionRef.current?.value ?? "").trim();
    try {
      if (isNew) {
        await api.insertComponent(loadedName, propSelects);
        const ns = await api.getComponentNames();
        setNames(ns);
      } else if (editable) {
        await api.updateComponentProperties(loadedName, propSelects);
      }
      await api.setComponentMedia(loadedName, imageBase64, descriptionValue);

      setIsNew(false);
      setDescription(descriptionValue);
      setDescriptionTouched(false);
      originalRef.current = { props: propSelects, imageBase64 };
      setDirty(false);
      onChanged();
      setInfo({ title: "Готово", text: "Изменения сохранены." });
    } catch (e) {
      setInfo({ title: "Ошибка", text: String(e) });
    }
  }

  async function handleDelete() {
    try {
      await api.deleteComponent(loadedName);
      const ns = await api.getComponentNames();
      setNames(ns);
      if (ns.length > 0) await loadComponent(ns[0]);
      else clearFields();
      onChanged();
      setInfo({ title: "Готово", text: "Компонент удалён." });
    } catch (e) {
      setInfo({ title: "Ошибка", text: String(e) });
    } finally {
      setConfirmDelete(false);
    }
  }
```

на:

```typescript
  async function handleSave() {
    const seen = new Set<number>();
    for (let i = 0; i < 4; i++) {
      const p = propSelects[i];
      if (p === null) {
        setInfo({ title: "Ошибка", text: `Выберите значение во всех 4 полях «Свойство» (поле ${i + 1}).` });
        return;
      }
      if (seen.has(p)) {
        setInfo({ title: "Ошибка", text: "Свойства компонента не должны повторяться." });
        return;
      }
      seen.add(p);
    }

    // Для не-пользовательских ингредиентов свойства/название всё равно
    // заблокированы в форме — тут реально может меняться только картинка
    // и описание, но это тоже правка "базовых" данных, поэтому отдельное
    // подтверждение.
    if (!isNew && !editable) {
      setConfirmBaseEdit(true);
      return;
    }
    await performSave();
  }

  async function performSave() {
    const descriptionValue = (descriptionRef.current?.value ?? "").trim();
    // propSelects прошли валидацию в handleSave (все 4 не null) до вызова
    // performSave (единственный вызывающий, включая путь через confirmBaseEdit).
    const propIds = propSelects as [number, number, number, number];
    try {
      let id = loadedId;
      if (isNew) {
        id = await api.insertComponent(loadedName, CURRENT_LANG, propIds);
        const ns = await api.getComponentNames(CURRENT_LANG);
        setNames(ns);
      } else if (editable && id !== null) {
        await api.updateComponentProperties(id, propIds);
      }
      if (id !== null) {
        await api.setComponentMedia(id, CURRENT_LANG, imageBase64, descriptionValue);
      }

      setLoadedId(id);
      setIsNew(false);
      setDescription(descriptionValue);
      setDescriptionTouched(false);
      originalRef.current = { props: propSelects, imageBase64 };
      setDirty(false);
      onChanged();
      setInfo({ title: "Готово", text: "Изменения сохранены." });
    } catch (e) {
      setInfo({ title: "Ошибка", text: String(e) });
    }
  }

  async function handleDelete() {
    if (loadedId === null) return;
    try {
      await api.deleteComponent(loadedId);
      const ns = await api.getComponentNames(CURRENT_LANG);
      setNames(ns);
      if (ns.length > 0) await loadComponent(ns[0].id, ns[0].name);
      else clearFields();
      onChanged();
      setInfo({ title: "Готово", text: "Компонент удалён." });
    } catch (e) {
      setInfo({ title: "Ошибка", text: String(e) });
    } finally {
      setConfirmDelete(false);
    }
  }
```

- [ ] **Step 4: Обновить JSX — Select'ы "Ингредиент" и "Свойство N"**

Замените:

```typescript
              <Select
                flex={1}
                data={names}
                value={names.includes(loadedName) ? loadedName : null}
                onChange={(v) => v && requestAction({ kind: "load", name: v })}
                searchable
                clearable={false}
                comboboxProps={{ withinPortal: true }}
              />
```

на:

```typescript
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
```

Замените:

```typescript
            <Select
              key={i}
              label={`Свойство ${i + 1}`}
              placeholder="— не выбрано —"
              data={allProperties}
              value={propSelects[i] || null}
              disabled={!editable}
              onChange={(v) => {
                const next = [...propSelects] as [string, string, string, string];
                next[i] = v ?? "";
                setPropSelects(next);
                recomputeDirty({ props: next });
              }}
              searchable
              clearable
              comboboxProps={{ withinPortal: true }}
            />
```

на:

```typescript
            <Select
              key={i}
              label={`Свойство ${i + 1}`}
              placeholder="— не выбрано —"
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
```

- [ ] **Step 5: Проверить компиляцию — весь проект должен быть зелёным**

Run: `npx tsc -b`
Expected: PASS, без ошибок во всём проекте.

Run: `npm run build`
Expected: PASS (`tsc -b && vite build` завершается без ошибок, `dist/`
собирается).

Run: `npm run lint`
Expected: PASS (oxlint не находит новых проблем; если найдёт
предупреждения по правилам, не связанным с этим планом — не чинить их в
рамках этой задачи, только новые ошибки, введённые этим планом).

- [ ] **Step 6: Commit**

```bash
git add src/components/EditorModal.tsx
git commit -m "feat: switch EditorModal.tsx to id-based selection (Task 5 of B2b)"
```

---

## После выполнения плана

Приложение снова полностью типобезопасно и рантайм-совместимо
(фронтенд/бэкенд формы `invoke()` совпадают) — B2 (переход на `id`)
полностью завершён. Реальное переключение языка (B3) и UX редактирования
базы на разных языках (B4) остаются отдельными будущими планами. Перед
тем как считать B2b полностью готовым, попросить пользователя
запустить `npm run tauri dev` самостоятельно и прислать
скриншот/подтверждение, что поиск сочетаний, "Показать свойства" и
"Редактировать базу" по-прежнему работают — эта задача сознательно не
поднимает dev-сервер и не пытается автоматизировать браузер (см. правило
проекта про визуальную проверку).
