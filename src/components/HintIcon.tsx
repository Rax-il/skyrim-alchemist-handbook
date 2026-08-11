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
  // Встроенный в кнопку вариант (rightSection) — фиксированный серый цвет
  // на некоторых темах кнопки читался как белый и терялся на фоне. Вместо
  // подбора цвета под каждую тему значок просто наследует цвет текста самой
  // кнопки (inline style: побеждает CSS-класс ActionIcon по специфичности).
  inheritColor?: boolean;
}

export function HintIcon({ label, inheritColor = false }: HintIconProps) {
  return (
    <Tooltip label={label} multiline w={260} withArrow>
      <ActionIcon
        variant="subtle"
        color={inheritColor ? undefined : "gray"}
        size="xs"
        onClick={(e) => e.stopPropagation()}
        aria-label="Подсказка"
        style={inheritColor ? { color: "inherit" } : undefined}
      >
        <IconHelpCircle size={14} />
      </ActionIcon>
    </Tooltip>
  );
}
