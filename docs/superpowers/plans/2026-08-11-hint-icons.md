# Подсказки-«вопросики» (HintIcon) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** добавить один переиспользуемый компонент `HintIcon` (значок «?» в
кружке с тултипом при наведении) и подключить его в 4 местах интерфейса, где
поведение сейчас неочевидно без пояснения.

**Architecture:** две последовательные задачи. Task 1 — сам компонент
`HintIcon` (самодостаточный, свой коммит). Task 2 — подключение во всех трёх
уже существующих файлах-потребителях (`EditorModal.tsx`, `SettingsModal.tsx`,
`ControlPanel.tsx`) одним заходом — суммарный диф небольшой (около 15 строк
на 3 файла), дробить по отдельному файлу на задачу было бы искусственным
разбиением без смысловой причины (ревьюер не может осмысленно принять один
файл-потребитель и отклонить другой — они меняются по одному и тому же
паттерну).

**Tech Stack:** TypeScript/React/Mantine (существующий стек проекта).
`@tabler/icons-react` (уже зависимость в `package.json`, версия ^3.46.0) —
используется впервые в `src/`, готового паттерна использования в проекте
ещё нет, `HintIcon.tsx` — первый. Тестового раннера на фронтенде в проекте
нет — верификация как в предыдущих двух планах (B2b, B3+B4): `npx tsc -b` +
`npm run build` + `npm run lint`.

## Global Constraints

- **Иконка — `IconHelpCircle`**, не `IconInfoCircle` — так пользователь сам
  описал элемент («вопросик», знак вопроса в кружке).
- **`onClick={(e) => e.stopPropagation()}` на самой иконке обязателен** —
  `HintIcon` используется и отдельно (у текстовой метки), и встроенно внутрь
  кликабельной кнопки через `rightSection` (Task 2, пункты 3-4) — без этого
  клик по значку внутри кнопки запускал бы саму кнопку. Наведение (hover)
  безопасно в любом случае — `onClick` кнопки не срабатывает от наведения.
  `stopPropagation` безвреден и в отдельностоящем варианте (родительского
  `onClick` там нет) — поэтому один компонент годится для обоих сценариев,
  два варианта не нужны.
- **Тексты подсказок — ровно как в дизайн-документе**, без перефразирования:
  1. `EditorModal.tsx`, «Описание»: «Корректировки описания ингредиента
     будут сохранены только для текущего языка»
  2. `SettingsModal.tsx`, «Макс. кол-во сочетаний»: «Количество комбинаций
     ингредиентов, отображаемое при нажатии на кнопки «Парные сочетания» и
     «Тройные сочетания»»
  3. `ControlPanel.tsx`, кнопка «Парные сочетания»: «Вывести сочетания двух
     ингредиентов, дающих максимальное количество эффектов. (Учитывает
     переключатель «Все»/«Улучшения»/«Яды».)»
  4. `ControlPanel.tsx`, кнопка «Тройные сочетания»: «Вывести сочетания трёх
     ингредиентов, дающих максимальное количество эффектов. (Учитывает
     переключатель «Все»/«Улучшения»/«Яды».)»
- **Кнопки в `ControlPanel.tsx` уже растянуты на всю ширину сайдбара**
  (родительский `Stack` растягивает детей по умолчанию) — добавление
  `rightSection` не меняет общую ширину кнопки, значок просто занимает часть
  уже существующего свободного места у правого края.
- **`InputWrapper.label` (и, следовательно, `TextInput.label`) принимает
  `React.ReactNode`**, не только строку — подтверждено чтением
  `node_modules/@mantine/core/lib/components/Input/InputWrapper/InputWrapper.d.ts`.
  Значит `label` в Task 2 можно заменить на `<Group>` без каких-либо иных
  изменений структуры компонента.

---

### Task 1: `src/components/HintIcon.tsx`

**Files:**
- Create: `src/components/HintIcon.tsx`

**Interfaces:**
- Produces (используется в Task 2 всеми тремя файлами-потребителями):
  - `export function HintIcon({ label }: { label: string }): JSX.Element`

- [ ] **Step 1: Создать компонент**

```tsx
// HintIcon.tsx — маленький значок "?" в кружке с тултипом при наведении
// (в разговоре — "вопросик"). Используется и отдельно рядом с текстовой
// меткой, и встроенно внутрь кнопки через rightSection — stopPropagation
// на onClick нужен именно для второго случая (иначе клик по значку внутри
// кнопки запускал бы саму кнопку); в первом случае он просто безвреден,
// поэтому один компонент годится для обоих сценариев размещения.

import { ActionIcon, Tooltip } from "@mantine/core";
import { IconHelpCircle } from "@tabler/icons-react";

interface HintIconProps {
  label: string;
}

export function HintIcon({ label }: HintIconProps) {
  return (
    <Tooltip label={label} multiline w={260} withArrow>
      <ActionIcon
        variant="subtle"
        color="gray"
        size="xs"
        onClick={(e) => e.stopPropagation()}
        aria-label="Подсказка"
      >
        <IconHelpCircle size={14} />
      </ActionIcon>
    </Tooltip>
  );
}
```

- [ ] **Step 2: Проверить компиляцию**

Run: `npx tsc -b`
Expected: PASS — `HintIcon.tsx` пока никем не используется, но сам по себе
должен быть без ошибок типов (проверить, что `@tabler/icons-react`
резолвится и `IconHelpCircle` действительно существует в этой версии
пакета).

- [ ] **Step 3: Commit**

```bash
git add src/components/HintIcon.tsx
git commit -m "feat: add HintIcon component (Task 1 of hint-icons)"
```

---

### Task 2: Подключить `HintIcon` в `EditorModal.tsx`, `SettingsModal.tsx`, `ControlPanel.tsx`

**Files:**
- Modify: `src/components/EditorModal.tsx`
- Modify: `src/components/SettingsModal.tsx`
- Modify: `src/components/ControlPanel.tsx`

**Interfaces:**
- Consumes: `HintIcon` из `./HintIcon` (Task 1).

- [ ] **Step 1: `EditorModal.tsx` — рядом с «Описание»**

Замените:

```tsx
        <div>
          <Group gap={6} align="baseline">
            <Text size="sm" fw={700}>
              Описание
            </Text>
            <Text size="xs" c="dimmed">
              (Только для текущего языка)
            </Text>
          </Group>
```

на:

```tsx
        <div>
          <Group gap={6} align="baseline">
            <Text size="sm" fw={700}>
              Описание
            </Text>
            <Text size="xs" c="dimmed">
              (Только для текущего языка)
            </Text>
            <HintIcon label="Корректировки описания ингредиента будут сохранены только для текущего языка" />
          </Group>
```

Добавьте импорт (сразу после `import { LANGUAGE_OPTIONS } from "../lib/languages";`):

```tsx
import { HintIcon } from "./HintIcon";
```

- [ ] **Step 2: `SettingsModal.tsx` — рядом с «Макс. кол-во сочетаний»**

Замените:

```tsx
              <TextInput
                label="Макс. кол-во сочетаний"
                value={maxCombinationsInput}
                onChange={(e) => setMaxCombinationsInput(e.currentTarget.value.replace(/\D/g, ""))}
                inputMode="numeric"
              />
```

на:

```tsx
              <TextInput
                label={
                  <Group gap={6} align="center">
                    <span>Макс. кол-во сочетаний</span>
                    <HintIcon label="Количество комбинаций ингредиентов, отображаемое при нажатии на кнопки «Парные сочетания» и «Тройные сочетания»" />
                  </Group>
                }
                value={maxCombinationsInput}
                onChange={(e) => setMaxCombinationsInput(e.currentTarget.value.replace(/\D/g, ""))}
                inputMode="numeric"
              />
```

Добавьте импорт (сразу после `import { LANGUAGE_OPTIONS } from "../lib/languages";`):

```tsx
import { HintIcon } from "./HintIcon";
```

- [ ] **Step 3: `ControlPanel.tsx` — на кнопках «Парные»/«Тройные сочетания»**

Замените:

```tsx
      <Button variant="light" onClick={onFindPairs}>
        Парные сочетания
      </Button>
      <Button variant="light" onClick={onFindMaxCombinations}>
        Тройные сочетания
      </Button>
```

на:

```tsx
      <Button
        variant="light"
        onClick={onFindPairs}
        rightSection={
          <HintIcon label="Вывести сочетания двух ингредиентов, дающих максимальное количество эффектов. (Учитывает переключатель «Все»/«Улучшения»/«Яды».)" />
        }
      >
        Парные сочетания
      </Button>
      <Button
        variant="light"
        onClick={onFindMaxCombinations}
        rightSection={
          <HintIcon label="Вывести сочетания трёх ингредиентов, дающих максимальное количество эффектов. (Учитывает переключатель «Все»/«Улучшения»/«Яды».)" />
        }
      >
        Тройные сочетания
      </Button>
```

Добавьте импорт (сразу после `import type { ComponentNameInfo, FilterKind, PropertyInfo } from "../lib/api";`):

```tsx
import { HintIcon } from "./HintIcon";
```

- [ ] **Step 4: Проверить компиляцию — весь проект должен быть зелёным**

Run: `npx tsc -b`
Expected: PASS, без ошибок во всём проекте.

Run: `npm run build`
Expected: PASS.

Run: `npm run lint`
Expected: PASS (новых предупреждений от этого плана быть не должно).

- [ ] **Step 5: Commit**

```bash
git add src/components/EditorModal.tsx src/components/SettingsModal.tsx src/components/ControlPanel.tsx
git commit -m "feat: wire HintIcon into EditorModal, SettingsModal, ControlPanel (Task 2 of hint-icons)"
```

---

## После выполнения плана

Все 4 подсказки из дизайн-документа на месте. Визуальная проверка — попросить
пользователя запустить `npm run tauri dev` самостоятельно и подтвердить: значки
видны, при наведении показывают правильный текст, клик по значку на кнопках
«Парные»/«Тройные сочетания» не запускает сам поиск (см. `stopPropagation`).
Эта задача сознательно не поднимает dev-сервер и не пытается автоматизировать
браузер (см. правило проекта про визуальную проверку).
