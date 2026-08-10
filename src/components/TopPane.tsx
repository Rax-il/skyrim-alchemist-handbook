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
