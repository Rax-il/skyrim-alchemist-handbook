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
}

export function HintIcon({ label }: HintIconProps) {
  return (
    <Tooltip label={label} multiline w={260} withArrow>
      <ActionIcon
        variant="subtle"
        color="gray"
        size="xs"
        onClick={(e) => e.stopPropagation()}
        aria-label="Подсказка"
      >
        <IconHelpCircle size={14} />
      </ActionIcon>
    </Tooltip>
  );
}
