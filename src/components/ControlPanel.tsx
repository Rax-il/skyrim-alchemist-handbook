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

  return (
    <Stack gap={4} p="sm">
      <Text size="sm" fw={700}>
        Поиск сочетаний
      </Text>

      {[0, 1, 2, 3].map((i) => (
        <Select
          key={i}
          label={`Свойство ${i + 1}`}
          placeholder={EMPTY_OPTION}
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
        label="Тип свойств"
        mt={4}
      >
        <Stack gap={4} mt={4}>
          <Radio value="" label="Все" />
          <Radio value="Улучшение" label="Улучшения" />
          <Radio value="Яд" label="Яды" />
        </Stack>
      </Radio.Group>

      <Button variant="light" mt="xs" onClick={onFindCombinations}>
        Найти сочетания
      </Button>

      <Divider my="sm" />

      <Text size="sm" fw={700}>
        Ингредиент ({componentNames.length})
      </Text>
      <Select
        placeholder={EMPTY_OPTION}
        data={componentOptions}
        value={componentSelect !== null ? String(componentSelect) : null}
        onChange={(v) => onComponentSelectChange(v !== null ? Number(v) : null)}
        searchable
        clearable
        comboboxProps={{ withinPortal: true }}
      />
      <Button variant="light" onClick={onShowProperties}>
        Показать свойства
      </Button>

      <Divider my="sm" />

      <Button variant="light" onClick={onFindPairs}>
        Парные сочетания
      </Button>
      <Button variant="light" onClick={onFindMaxCombinations}>
        Тройные сочетания
      </Button>
    </Stack>
  );
}
