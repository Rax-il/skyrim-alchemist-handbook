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
          value={selects[i] || null}
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
        value={componentSelect || null}
        onChange={(v) => onComponentSelectChange(v ?? "")}
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
