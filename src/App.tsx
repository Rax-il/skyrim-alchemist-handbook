import { useCallback, useEffect, useRef, useState } from "react";
import { Button, Group, Menu, Modal, Stack, Text } from "@mantine/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api, CURRENT_LANG } from "./lib/api";
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

interface Props {
  appTheme: AppThemeName;
  onAppThemeChange: (theme: AppThemeName) => void;
  scale: AppScaleName;
  onScaleChange: (scale: AppScaleName) => void;
}

export default function App({ appTheme, onAppThemeChange, scale, onScaleChange }: Props) {
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
  const [resultsHeader, setResultsHeader] = useState("Найдено 0 комбинаций");

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

  // --- Начальная загрузка: список свойств, сохранённая раскладка (список
  // компонентов, ограниченный дополнениями, подтягивается отдельным
  // эффектом ниже — он же сработает повторно, когда сохранённый набор
  // дополнений подгрузится из getLayout и отличается от дефолтного) ---
  useEffect(() => {
    api.getProperties(CURRENT_LANG).then(setProperties);
    api.getLayout().then((l) => {
      setSidePanelWidth(l.side_panel_width);
      setSplitRatio(l.split_ratio);
      setEnabledAddons(l.enabled_addons);
      setMaxCombinations(l.max_combinations);
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

  // --- Список ингредиентов для выбора компонента, ограниченный включёнными
  // дополнениями. Срабатывает и при первой загрузке (с дефолтным "все
  // включены"), и при каждом применении новых настроек в SettingsModal.
  // Прежний выбранный компонент и результаты предыдущего поиска могли
  // ссылаться на теперь скрытые ингредиенты — сбрасываем оба, чтобы не
  // показывать стухшие данные; пользователь просто ищет заново.
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
      text: "Данная операция может занять некоторое время. Продолжать?",
      onConfirm: () => {
        action();
      },
    });
  }

  function setPlainResults(items: string[], emptyMessage: string) {
    setBottomMode({ kind: "list" });
    setResultsHeader(`Найдено ${items.length} комбинаций`);
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
    setResultsHeader(`Отображено ${displayedCount} комбинаций`);
    setResults(lines.length === 0 ? [emptyMessage] : lines);
  }

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
            <button className="menu-bar-button">Файл</button>
          </Menu.Target>
          <Menu.Dropdown>
            <Menu.Item onClick={() => setEditorOpen(true)}>Редактировать базу</Menu.Item>
            <Menu.Item onClick={() => setSettingsOpen(true)}>Настройки</Menu.Item>
            <Menu.Divider />
            <Menu.Item onClick={handleExit}>Выход</Menu.Item>
          </Menu.Dropdown>
        </Menu>
        <Menu shadow="md" width={200}>
          <Menu.Target>
            <button className="menu-bar-button">О программе</button>
          </Menu.Target>
          <Menu.Dropdown>
            <Menu.Item onClick={() => setAboutOpen(true)}>О программе</Menu.Item>
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
        enabledAddons={enabledAddons}
        onEnabledAddonsChange={setEnabledAddons}
        maxCombinations={maxCombinations}
        onMaxCombinationsChange={setMaxCombinations}
      />

      <Modal opened={info !== null} onClose={() => setInfo(null)} title={info?.title} size="sm">
        <Text size="sm">{info?.text}</Text>
      </Modal>

      <Modal opened={confirm !== null} onClose={() => setConfirm(null)} title="Подтверждение" size="sm">
        <Stack gap="md">
          <Text size="sm">{confirm?.text}</Text>
          <Group justify="flex-end" gap="sm">
            <Button variant="default" onClick={() => setConfirm(null)}>
              Отмена
            </Button>
            <Button
              onClick={() => {
                confirm?.onConfirm();
                setConfirm(null);
              }}
            >
              Продолжить
            </Button>
          </Group>
        </Stack>
      </Modal>
    </div>
  );
}
