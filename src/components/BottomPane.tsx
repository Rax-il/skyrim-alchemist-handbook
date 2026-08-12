import { Group, Image, ScrollArea, Stack, Text } from "@mantine/core";
import { useTranslation } from "react-i18next";
import { GOOD_QUALITY_SIZE, useAdaptiveImageSize } from "../lib/useAdaptiveImageSize";

export type BottomMode =
  | { kind: "list" }
  | { kind: "media"; name: string; description: string; imageDataUrl: string | null };

interface Props {
  header: string;
  results: string[];
  mode: BottomMode;
}

// Контейнер всегда фиксированного размера (GOOD_QUALITY_SIZE) — как в
// draw_centered_preview из исходного main.rs. Сама картинка внутри него
// рисуется в РЕАЛЬНОМ вычисленном размере (см. useAdaptiveImageSize), а не
// вписывается по пропорциям — так "плохие" миниатюры с вики (обычно 50×50)
// не растягиваются на весь контейнер и не превращаются в кашу из пикселей.
const PREVIEW_BOX = GOOD_QUALITY_SIZE;

export function BottomPane({ header, results, mode }: Props) {
  const { t } = useTranslation();
  const imageDataUrl = mode.kind === "media" ? mode.imageDataUrl : null;
  const imageSize = useAdaptiveImageSize(imageDataUrl);

  return (
    <Stack gap={0} h="100%">
      <Text size="sm" fw={700} c={mode.kind === "list" ? "blue" : undefined} p="xs" pb={4}>
        {header}
      </Text>

      {mode.kind === "list" && (
        <ScrollArea flex={1} px="xs">
          <Stack gap={2}>
            {results.map((line, i) =>
              line === "" ? (
                <div key={i} style={{ height: 10 }} />
              ) : (
                // Строки с отступом — отдельные эффекты внутри сочетания
                // (см. find_pairs/find_max_combinations), а не сама формула
                // сочетания — их не выделяем жирным, в отличие от формулы.
                <Text key={i} size="sm" fw={line.startsWith("    ") ? undefined : 700}>
                  {line}
                </Text>
              ),
            )}
          </Stack>
        </ScrollArea>
      )}

      {mode.kind === "media" && (
        <Group align="flex-start" wrap="nowrap" flex={1} px="xs" gap="md">
          <div
            style={{
              width: PREVIEW_BOX,
              height: PREVIEW_BOX,
              minWidth: PREVIEW_BOX,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            {mode.imageDataUrl ? (
              <Image src={mode.imageDataUrl} w={imageSize} h={imageSize} fit="contain" />
            ) : (
              <Text c="dimmed">{t("common.noImage")}</Text>
            )}
          </div>
          <ScrollArea flex={1} h="100%">
            <Text size="sm" style={{ whiteSpace: "pre-wrap" }}>
              {mode.description}
            </Text>
          </ScrollArea>
        </Group>
      )}
    </Stack>
  );
}
