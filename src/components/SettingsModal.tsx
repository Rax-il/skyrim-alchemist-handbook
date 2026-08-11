import { useEffect, useState } from "react";
import {
  Button,
  Checkbox,
  Group,
  Modal,
  ScrollArea,
  Select,
  Stack,
  Text,
  TextInput,
  useMantineColorScheme,
} from "@mantine/core";
import type { MantineColorScheme } from "@mantine/core";
import { getCurrentWindow, LogicalSize } from "@tauri-apps/api/window";
import { api } from "../lib/api";
import {
  BASE_WINDOW_HEIGHT,
  BASE_WINDOW_WIDTH,
  MIN_WINDOW_HEIGHT,
  SCALE_FACTOR_BY_NAME,
} from "../lib/appTheme";
import type { AppScaleName, AppThemeName } from "../lib/appTheme";
import { ADDON_CHECKBOX_IDS, ADDON_LABELS } from "../lib/addons";
import type { AddonId } from "../lib/addons";
import { LANGUAGE_OPTIONS } from "../lib/languages";

interface Props {
  opened: boolean;
  onClose: () => void;
  appTheme: AppThemeName;
  onAppThemeChange: (theme: AppThemeName) => void;
  scale: AppScaleName;
  onScaleChange: (scale: AppScaleName) => void;
  currentLanguage: string;
  onLanguageChange: (lang: string) => void;
  enabledAddons: AddonId[];
  onEnabledAddonsChange: (addons: AddonId[]) => void;
  maxCombinations: number;
  onMaxCombinationsChange: (maxCombinations: number) => void;
}

const DEFAULT_MAX_COMBINATIONS = 100;

const SETTINGS_WIDTH = 500;
const SETTINGS_HEIGHT = 350;
const FOOTER_HEIGHT = 56;
// Левая и правая части — поровну.
const LEFT_WIDTH = SETTINGS_WIDTH / 2;
const RIGHT_WIDTH = SETTINGS_WIDTH - LEFT_WIDTH;

// Modal size={SETTINGS_WIDTH} у Mantine сам масштабируется через
// --mantine-scale (числовые size-пропы идут через их rem()). Наша
// собственная вёрстка внутри — обычные px, поэтому её нужно масштабировать
// вручную той же переменной, иначе при смене масштаба модалка и её
// внутренняя разметка разъезжаются (модалка сама уменьшается/увеличивается,
// а фиксированные px внутри — нет).
function scaled(px: number): string {
  return `calc(${px}px * var(--mantine-scale))`;
}

const SCALE_OPTIONS: AppScaleName[] = ["Мелкий", "Нормальный", "Крупный"];
const THEME_OPTIONS = ["Системная", "Светлая", "Тёмная", "Skyrim"];
const SKYRIM_LABEL = "Skyrim";

// Соответствие подписей в списке и значений MantineColorScheme.
const THEME_LABEL_BY_SCHEME: Record<MantineColorScheme, string> = {
  auto: "Системная",
  light: "Светлая",
  dark: "Тёмная",
};
const THEME_SCHEME_BY_LABEL: Record<string, MantineColorScheme> = {
  Системная: "auto",
  Светлая: "light",
  Тёмная: "dark",
};

export function SettingsModal({
  opened,
  onClose,
  appTheme,
  onAppThemeChange,
  scale: appScale,
  onScaleChange,
  currentLanguage,
  onLanguageChange,
  enabledAddons,
  onEnabledAddonsChange,
  maxCombinations,
  onMaxCombinationsChange,
}: Props) {
  const { colorScheme, setColorScheme } = useMantineColorScheme();

  const [scale, setScale] = useState<AppScaleName>(appScale);
  const [language, setLanguage] = useState<string>(currentLanguage);
  const [theme, setTheme] = useState<string>(
    appTheme === "skyrim" ? SKYRIM_LABEL : THEME_LABEL_BY_SCHEME[colorScheme],
  );
  const [checkedAddons, setCheckedAddons] = useState<AddonId[]>(enabledAddons);
  // Текстовое поле — храним как строку (ограничена только цифрами через
  // onChange), а не число, чтобы не мешать вводу (например, стереть всё и
  // напечатать заново).
  const [maxCombinationsInput, setMaxCombinationsInput] = useState(String(maxCombinations));
  const [languageWarningOpen, setLanguageWarningOpen] = useState(false);

  // При каждом открытии окна — подхватить реально применённые сейчас тему,
  // масштаб, язык и набор дополнений (а не то, что было выбрано в списке,
  // но не применено).
  useEffect(() => {
    if (opened) {
      setTheme(appTheme === "skyrim" ? SKYRIM_LABEL : THEME_LABEL_BY_SCHEME[colorScheme]);
      setScale(appScale);
      setLanguage(currentLanguage);
      setCheckedAddons(enabledAddons);
      setMaxCombinationsInput(String(maxCombinations));
    }
  }, [opened, appTheme, colorScheme, appScale, currentLanguage, enabledAddons, maxCombinations]);

  // Смена языка — единственная настройка, которая может скрыть уже
  // существующие данные пользователя (его собственные ингредиенты, видимые
  // только на языке создания, см. design doc, раздел B3+B4). Если язык
  // реально меняется и у пользователя есть хотя бы один такой ингредиент —
  // сначала спрашиваем подтверждение, весь остальной Apply откладывается до
  // ответа.
  function handleApply() {
    if (language !== currentLanguage) {
      api.hasUserAddedComponents().then((has) => {
        if (has) {
          setLanguageWarningOpen(true);
        } else {
          applyAll();
        }
      });
      return;
    }
    applyAll();
  }

  function applyAll() {
    if (theme === SKYRIM_LABEL) {
      onAppThemeChange("skyrim");
    } else {
      onAppThemeChange("default");
      setColorScheme(THEME_SCHEME_BY_LABEL[theme] ?? "auto");
    }

    // Ресайзим окно, только если масштаб реально поменяли — иначе
    // "Применить" ради темы/языка/дополнений насильно возвращало бы окно к
    // дефолтному для текущего масштаба размеру, затирая ручной ресайз
    // пользователя (см. баг: окно "прыгало" при любом Apply).
    if (scale !== appScale) {
      const factor = SCALE_FACTOR_BY_NAME[scale];
      const height = Math.max(BASE_WINDOW_HEIGHT * factor, MIN_WINDOW_HEIGHT);
      getCurrentWindow().setSize(new LogicalSize(BASE_WINDOW_WIDTH * factor, height));
      api.saveScale(scale);
    }
    onScaleChange(scale);

    if (language !== currentLanguage) {
      onLanguageChange(language);
      api.saveLanguage(language);
    }

    onEnabledAddonsChange(checkedAddons);
    api.saveAddons(checkedAddons);

    const parsedMax = parseInt(maxCombinationsInput, 10);
    const nextMax = Number.isFinite(parsedMax) && parsedMax >= 1 ? parsedMax : DEFAULT_MAX_COMBINATIONS;
    onMaxCombinationsChange(nextMax);
    api.saveMaxCombinations(nextMax);

    onClose();
  }

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      title="Настройки"
      padding={0}
      size={SETTINGS_WIDTH}
      centered
      styles={{
        content: { overflow: "hidden" },
        header: { padding: "8px 12px" },
        body: { padding: 0 },
      }}
    >
      <div style={{ width: "100%", display: "flex", flexDirection: "column" }}>
        <div
          style={{ width: "100%", height: scaled(SETTINGS_HEIGHT), display: "flex" }}
        >
          {/* Левая часть — раскрывающиеся списки: Язык, Масштаб, Цветовая тема. */}
          <div style={{ width: scaled(LEFT_WIDTH), flexShrink: 0, padding: scaled(16) }}>
            <Stack gap="md">
              <Select
                label="Язык"
                data={LANGUAGE_OPTIONS}
                value={language}
                onChange={(v) => setLanguage(v ?? language)}
                allowDeselect={false}
                comboboxProps={{ withinPortal: true }}
              />
              <Select
                label="Масштаб"
                data={SCALE_OPTIONS}
                value={scale}
                onChange={(v) => setScale((v as AppScaleName | null) ?? scale)}
                allowDeselect={false}
                comboboxProps={{ withinPortal: true }}
              />
              <Select
                label="Цветовая тема"
                data={THEME_OPTIONS}
                value={theme}
                onChange={(v) => setTheme(v ?? theme)}
                allowDeselect={false}
                comboboxProps={{ withinPortal: true }}
              />
              <TextInput
                label="Макс. кол-во сочетаний"
                value={maxCombinationsInput}
                onChange={(e) => setMaxCombinationsInput(e.currentTarget.value.replace(/\D/g, ""))}
                inputMode="numeric"
              />
            </Stack>
          </div>

          {/* Правая часть — список чекбоксов с источниками ингредиентов. */}
          <div
            style={{
              width: scaled(RIGHT_WIDTH),
              flexShrink: 0,
              borderLeft: "1px solid var(--mantine-color-gray-3)",
              display: "flex",
              flexDirection: "column",
            }}
          >
            <Text size="xs" fw={700} p="xs" pb={4}>
              Дополнения к игре
            </Text>
            <ScrollArea flex={1} px="xs">
              <Stack gap={6}>
                {ADDON_CHECKBOX_IDS.map((id) => (
                  <Checkbox
                    key={id}
                    size="xs"
                    color={id === "base_game" ? "yellow" : undefined}
                    label={ADDON_LABELS[id]}
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
          </div>
        </div>

        {/* Нижняя панель — Применить / Отмена. */}
        <Group
          justify="flex-end"
          gap="sm"
          style={{
            height: scaled(FOOTER_HEIGHT),
            borderTop: "1px solid var(--mantine-color-gray-3)",
            paddingInline: scaled(16),
          }}
        >
          <Button variant="default" onClick={onClose}>
            Отмена
          </Button>
          <Button onClick={handleApply}>Применить</Button>
        </Group>
      </div>

      <Modal
        opened={languageWarningOpen}
        onClose={() => setLanguageWarningOpen(false)}
        title="Подтверждение"
        size="sm"
      >
        <Text size="sm" mb="md">
          У вас есть ингредиенты, добавленные вручную — они видны только на
          языке, на котором были созданы. После смены языка они пропадут из
          списков (ничего не удаляется — снова появятся, если вернуться на
          прежний язык). Продолжить?
        </Text>
        <Group justify="flex-end">
          <Button variant="default" onClick={() => setLanguageWarningOpen(false)}>
            Отмена
          </Button>
          <Button
            onClick={() => {
              setLanguageWarningOpen(false);
              applyAll();
            }}
          >
            Продолжить
          </Button>
        </Group>
      </Modal>
    </Modal>
  );
}
