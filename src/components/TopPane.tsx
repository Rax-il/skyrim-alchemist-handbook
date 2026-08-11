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
