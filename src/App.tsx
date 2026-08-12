import { useCallback, useEffect, useRef, useState } from "react";
import { Button, Group, Menu, Modal, Stack, Text } from "@mantine/core";
import { IconDoorExit, IconHelpCircle, IconPencil, IconSettings } from "@tabler/icons-react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useTranslation } from "react-i18next";
import i18n from "./i18n";
import { api } from "./lib/api";
import type { ComponentNameInfo, CombinationResult, FilterKind, PropertyInfo } from "./lib/api";
import { useDragHandle } from "./lib/useDrag";
import { ControlPanel } from "./components/ControlPanel";
import { TopPane } from "./components/TopPane";
import type { TopMode } from "./components/TopPane";
import { BottomPane } from "./components/BottomPane";
import type { BottomMode } from "./components/BottomPane";
import { EditorModal } from "./components/EditorModal";
import { AboutModal } from "./components/AboutModal";
import { ThanksModal } from "./components/ThanksModal";
import { SettingsModal } from "./components/SettingsModal";
import type { AppScaleName, AppThemeName } from "./lib/appTheme";
import { ALL_ADDON_IDS } from "./lib/addons";
import type { AddonId } from "./lib/addons";

const HANDLE_SIZE = 8;
const DEFAULT_MAX_COMBINATIONS = 100;
const DEFAULT_LANGUAGE = "en";

interface Props {
  appTheme: AppThemeName;
  onAppThemeChange: (theme: AppThemeName) => void;
  scale: AppScaleName;
  onScaleChange: (scale: AppScaleName) => void;
}

export default function App({ appTheme, onAppThemeChange, scale, onScaleChange }: Props) {
  const { t } = useTranslation();
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

  const [results, setResults] = useState<string[]>([]);
  const [resultsHeader, setResultsHeader] = useState(() => t("app.resultsFound", { count: 0 }));

  const [topMode, setTopMode] = useState<TopMode>({ kind: "empty" });
  const [bottomMode, setBottomMode] = useState<BottomMode>({ kind: "list" });

  const lastCombosRef = useRef<CombinationResult[]>([]);
  const [enabledComponents, setEnabledComponents] = useState<Record<number, boolean>>({});

  const [editorOpen, setEditorOpen] = useState(false);
  const [aboutOpen, setAboutOpen] = useState(false);
  const [thanksOpen, setThanksOpen] = useState(false);
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [info, setInfo] = useState<{ title: string; text: string } | null>(null);
  const [confirm, setConfirm] = useState<{ text: string; onConfirm: () => void } | null>(null);

  const [sidePanelWidth, setSidePanelWidth] = useState(300);
  const [splitRatio, setSplitRatio] = useState(0.25);
  const containerRef = useRef<HTMLDivElement>(null);

  // Пока сохранённые настройки ещё не подгрузились — считаем, что включены
  // все дополнения (так же ведёт себя бэкенд для повреждённого/отсутствующего
  // settings.json, см. layout::default_addons).
  const [enabledAddons, setEnabledAddons] = useState<AddonId[]>(ALL_ADDON_IDS);
  // Ограничение числа результатов "Парных"/"Тройных сочетаний" (см.
  // SettingsModal). Дефолт — пока сохранённые настройки не подгрузились.
  const [maxCombinations, setMaxCombinations] = useState(DEFAULT_MAX_COMBINATIONS);
  // Текущий язык (план B3) — раньше был захардкоженной константой CURRENT_LANG.
  const [language, setLanguage] = useState(DEFAULT_LANGUAGE);

  // --- Начальная загрузка: сохранённая раскладка (список свойств и список
  // компонентов, ограниченный дополнениями и языком, подтягиваются отдельным
  // эффектом ниже — он же сработает повторно, когда сохранённые дополнения
  // и/или язык подгрузятся из getLayout и отличаются от дефолтных) ---
  useEffect(() => {
    api.getLayout().then((l) => {
      setSidePanelWidth(l.side_panel_width);
      setSplitRatio(l.split_ratio);
      setEnabledAddons(l.enabled_addons);
      setMaxCombinations(l.max_combinations);
      setLanguage(l.language);
      // Новая установка (settings.json ещё не было) — открываем "Настройки"
      // сразу, чтобы пользователь на нерусском языке сходу увидел выбор
      // языка, а не искал его в меню сам. Флаг помечается сразу, а не при
      // закрытии окна, — иначе при выходе без "Применить" оно всплывало бы
      // на каждом следующем запуске.
      if (!l.settings_shown) {
        setSettingsOpen(true);
        api.saveSettingsShown();
      }
    });
  }, []);

  // --- Сохранение раскладки при изменении (с задержкой, чтобы не писать
  // файл на каждый пиксель при перетаскивании разделителя) ---
  const saveTimer = useRef<number | null>(null);
  useEffect(() => {
    if (saveTimer.current) window.clearTimeout(saveTimer.current);
    saveTimer.current = window.setTimeout(() => {
      api.saveLayout({ side_panel_width: sidePanelWidth, split_ratio: splitRatio });
    }, 400);
  }, [sidePanelWidth, splitRatio]);

  // --- Синхронизация языка интерфейса с i18next + заголовок окна (Tauri
  // window title, не React DOM — не переводится через JSX) + список свойств
  // и список ингредиентов, ограниченный включёнными дополнениями и текущим
  // языком. Срабатывает при первой загрузке (с дефолтными значениями), при
  // каждом применении новых настроек в SettingsModal (дополнения ИЛИ язык)
  // и при смене языка через Настройки. Прежний выбранный компонент и
  // результаты предыдущего поиска могли ссылаться на теперь скрытые
  // ингредиенты (другой набор дополнений или язык, в котором их вообще нет
  // — см. design doc про пользовательские ингредиенты) — сбрасываем оба,
  // чтобы не показывать стухшие данные; пользователь просто ищет заново.
  // Важно: i18n.changeLanguage — асинхронная операция, а t()/i18n.t() ниже
  // должны звать её ПОСЛЕ переключения языка (иначе строки вроде
  // resultsHeader вычисляются ещё на предыдущем языке и повисают в таком
  // виде в state до следующего реального обновления данных — был баг,
  // пойманный пользователем на скриншоте).
  useEffect(() => {
    i18n.changeLanguage(language).then(() => {
      getCurrentWindow().setTitle(i18n.t("appTitle"));
      api.getProperties(language).then(setProperties);
      api.getComponentNamesFiltered(enabledAddons, language).then((names) => {
        setComponentNames(names);
        setComponentSelect((prev) => (prev !== null && names.some((n) => n.id === prev) ? prev : null));
      });
      lastCombosRef.current = [];
      setEnabledComponents({});
      setTopMode({ kind: "empty" });
      setBottomMode({ kind: "list" });
      setResults([]);
      setResultsHeader(i18n.t("app.resultsFound", { count: 0 }));
    });
  }, [enabledAddons, language]);

  function refreshLists() {
    api.getProperties(language).then(setProperties);
    api.getComponentNamesFiltered(enabledAddons, language).then(setComponentNames);
  }

  function info_(title: string, text: string) {
    setInfo({ title, text });
  }

  // Перебор пар/троек по всем включённым ингредиентам может занимать
  // заметное время (особенно "Тройные сочетания") — предупреждаем перед
  // запуском и стартуем сам поиск только после подтверждения. Индикатор
  // хода выполнения (и с реальными событиями, и просто статичный спиннер)
  // пробовали и отказались — на этой связке Tauri/WebKitGTK окно вообще не
  // перерисовывается, пока не вернётся ответ на invoke(), так что от любой
  // визуальной индикации процесса толку не было — только это предупреждение.
  function confirmThenRun(action: () => Promise<void>) {
    setConfirm({
      text: t("app.slowOperationConfirm"),
      onConfirm: () => {
        action();
      },
    });
  }

  function setPlainResults(items: string[], emptyMessage: string) {
    setBottomMode({ kind: "list" });
    setResultsHeader(t("app.resultsFound", { count: items.length }));
    setResults(items.length === 0 ? [emptyMessage] : items);
  }

  // "Парные"/"Тройные сочетания" возвращают не список отдельных строк-комбо
  // (как find_combinations), а построчный текст вида find_pairs — одна
  // запись = блок строк (заголовок + эффекты с отступом), блоки разделены
  // пустой строкой. items.length там — число строк, а не число сочетаний,
  // поэтому число реально показанных записей считаем отдельно; оно и так не
  // может превышать "Макс. кол-во сочетаний" из Настроек — бэкенд уже
  // обрезал список до этого лимита перед тем, как его вернуть.
  function setBlockResults(lines: string[], emptyMessage: string) {
    const displayedCount = lines.length === 0 ? 0 : lines.filter((l) => l === "").length + 1;
    setBottomMode({ kind: "list" });
    setResultsHeader(t("app.resultsDisplayed", { count: displayedCount }));
    setResults(lines.length === 0 ? [emptyMessage] : lines);
  }

  function applyComponentFilter(enabled: Record<number, boolean>) {
    const filtered = lastCombosRef.current
      .filter((c) => c.components.every((id) => enabled[id] ?? false))
      .map((c) => c.line);
    setPlainResults(filtered, t("app.noCombinationsFound"));
  }

  async function handleFindCombinations() {
    const chosen = selects.filter((s): s is number => s !== null);
    if (chosen.length === 0) {
      info_(t("common.attention"), t("app.errorSelectProperty"));
      return;
    }
    try {
      const found = await api.findCombinations(chosen, filter, enabledAddons, language);
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
      info_(t("common.error"), String(e));
    }
  }

  async function handleShowProperties() {
    if (componentSelect === null) {
      info_(t("common.attention"), t("app.errorSelectComponent"));
      return;
    }
    try {
      const props = await api.getComponentProperties(componentSelect, language);
      const selectedName = componentNames.find((c) => c.id === componentSelect)?.name ?? "";
      lastCombosRef.current = [];
      setEnabledComponents({});
      setTopMode({ kind: "properties", props });
      setResultsHeader(selectedName);

      const media = await api.getComponentMedia(componentSelect, language);
      setBottomMode({
        kind: "media",
        name: selectedName,
        description: media.description,
        imageDataUrl: media.image_data_url,
      });
    } catch (e) {
      info_(t("common.error"), String(e));
    }
  }

  async function handleFindPairs() {
    try {
      const found = await api.findPairs(filter, enabledAddons, maxCombinations, language);
      lastCombosRef.current = [];
      setEnabledComponents({});
      setTopMode({ kind: "checklist", items: [] });
      setBlockResults(found, t("app.noCombinationsFound"));
    } catch (e) {
      info_(t("common.error"), String(e));
    }
  }

  async function handleFindMaxCombinations() {
    try {
      const found = await api.findMaxCombinations(filter, enabledAddons, maxCombinations, language);
      lastCombosRef.current = [];
      setEnabledComponents({});
      setTopMode({ kind: "checklist", items: [] });
      setBlockResults(found, t("app.noCombinationsFound"));
    } catch (e) {
      info_(t("common.error"), String(e));
    }
  }

  function handleToggleComponent(id: number, checked: boolean) {
    const next = { ...enabledComponents, [id]: checked };
    setEnabledComponents(next);
    applyComponentFilter(next);
  }

  // --- Перетаскивание боковой панели (аналог resizable SidePanel в egui) ---
  const sideDrag = useDragHandle((dx) => {
    setSidePanelWidth((w) => Math.max(200, Math.min(600, w + dx)));
  });

  // --- Перетаскивание разделителя верх/низ (аналог VSplit handle) ---
  const splitDrag = useDragHandle((_dx, dy) => {
    const total = containerRef.current?.clientHeight ?? 600;
    const usable = Math.max(total - HANDLE_SIZE, 60);
    setSplitRatio((r) => Math.max(0.05, Math.min(0.95, r + dy / usable)));
  });

  const handleExit = useCallback(() => {
    getCurrentWindow().close();
  }, []);

  return (
    <div style={{ display: "flex", flexDirection: "column", height: "100vh", width: "100vw" }}>
      {/* Строка меню — аналог egui::TopBottomPanel::top("menu_bar") */}
      <div
        style={{
          display: "flex",
          borderBottom: "1px solid var(--mantine-color-gray-3)",
          padding: "2px 6px",
          gap: 4,
          flexShrink: 0,
        }}
      >
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
      </div>

      <div style={{ display: "flex", flex: 1, minHeight: 0 }}>
        {/* Боковая панель */}
        <div
          style={{
            width: sidePanelWidth,
            flexShrink: 0,
            overflowY: "auto",
            borderRight: "1px solid var(--mantine-color-gray-3)",
          }}
        >
          <ControlPanel
            properties={properties}
            selects={selects}
            onSelectsChange={setSelects}
            filter={filter}
            onFilterChange={setFilter}
            componentNames={componentNames}
            componentSelect={componentSelect}
            onComponentSelectChange={setComponentSelect}
            onFindCombinations={handleFindCombinations}
            onShowProperties={handleShowProperties}
            onFindPairs={() => confirmThenRun(handleFindPairs)}
            onFindMaxCombinations={() => confirmThenRun(handleFindMaxCombinations)}
          />
        </div>
        {/* Ручка изменения ширины боковой панели */}
        <div
          onMouseDown={sideDrag}
          style={{ width: 4, cursor: "col-resize", flexShrink: 0, background: "transparent" }}
        />

        {/* Основная область: верхняя панель / разделитель / нижняя панель */}
        <div ref={containerRef} style={{ flex: 1, display: "flex", flexDirection: "column", minWidth: 0 }}>
          <div
            style={{
              flexGrow: splitRatio,
              flexBasis: 0,
              minHeight: 40,
              overflow: "hidden",
              border: "1px solid var(--mantine-color-gray-3)",
              borderRadius: 4,
              margin: 4,
            }}
          >
            <TopPane mode={topMode} enabledComponents={enabledComponents} onToggleComponent={handleToggleComponent} />
          </div>

          <div
            onMouseDown={splitDrag}
            style={{
              height: HANDLE_SIZE,
              cursor: "row-resize",
              flexShrink: 0,
              display: "flex",
              alignItems: "center",
            }}
          >
            <div style={{ height: 2, width: "100%", background: "var(--mantine-color-gray-4)" }} />
          </div>

          <div
            style={{
              flexGrow: 1 - splitRatio,
              flexBasis: 0,
              minHeight: 40,
              overflow: "hidden",
              border: "1px solid var(--mantine-color-gray-3)",
              borderRadius: 4,
              margin: 4,
            }}
          >
            <BottomPane header={resultsHeader} results={results} mode={bottomMode} />
          </div>
        </div>
      </div>

      <EditorModal
        opened={editorOpen}
        onClose={() => setEditorOpen(false)}
        onChanged={() => {
          refreshLists();
        }}
        lang={language}
      />

      <AboutModal
        opened={aboutOpen}
        onClose={() => setAboutOpen(false)}
        onDonateClick={() => {
          setAboutOpen(false);
          setThanksOpen(true);
        }}
      />

      <ThanksModal opened={thanksOpen} onClose={() => setThanksOpen(false)} />

      <SettingsModal
        opened={settingsOpen}
        onClose={() => setSettingsOpen(false)}
        appTheme={appTheme}
        onAppThemeChange={onAppThemeChange}
        scale={scale}
        onScaleChange={onScaleChange}
        currentLanguage={language}
        onLanguageChange={setLanguage}
        enabledAddons={enabledAddons}
        onEnabledAddonsChange={setEnabledAddons}
        maxCombinations={maxCombinations}
        onMaxCombinationsChange={setMaxCombinations}
      />

      <Modal opened={info !== null} onClose={() => setInfo(null)} title={info?.title} size="sm">
        <Text size="sm">{info?.text}</Text>
      </Modal>

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
    </div>
  );
}
