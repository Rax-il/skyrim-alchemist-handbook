import { Button, Divider, Radio, Select, Stack, Text } from "@mantine/core";
import { useTranslation } from "react-i18next";
import type { ComponentNameInfo, FilterKind, PropertyInfo } from "../lib/api";
import { HintIcon } from "./HintIcon";

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
  const { t } = useTranslation();
  const propertyOptions = properties.map((p) => ({ value: String(p.id), label: p.name }));
  const componentOptions = componentNames.map((c) => ({ value: String(c.id), label: c.name }));

  const setSelect = (i: number, value: string | null) => {
    const next = [...selects] as Selects;
    next[i] = value !== null ? Number(value) : null;
    onSelectsChange(next);
  };

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
}
