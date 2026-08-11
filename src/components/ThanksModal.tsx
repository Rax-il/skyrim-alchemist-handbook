import { CloseButton, Modal, Text } from "@mantine/core";
import portrait from "../assets/thanks-portrait.png";

interface Props {
  opened: boolean;
  onClose: () => void;
}

const THANKS_WIDTH = 490;
const THANKS_HEIGHT = 400;
// Пока нет реального QR — просто квадрат-заглушка на его месте.
const QR_PLACEHOLDER_SIZE = 140;
// Заглушка под баннер сервиса (заменит текст "Донат осуществляется...") —
// ширина как у QR-заглушки выше, высота втрое меньше.
const SERVICE_BANNER_HEIGHT = QR_PLACEHOLDER_SIZE / 3;
// Смещение разделителя картинка/текст от центра (вправо — положительное).
const DIVIDER_OFFSET = 40;

export function ThanksModal({ opened, onClose }: Props) {
  return (
    <Modal
      opened={opened}
      onClose={onClose}
      withCloseButton={false}
      padding={0}
      size={THANKS_WIDTH}
      centered
      styles={{
        content: { overflow: "hidden" },
        body: { padding: 0 },
      }}
    >
      <div style={{ position: "relative", width: "100%", height: THANKS_HEIGHT, display: "flex" }}>
        <CloseButton
          onClick={onClose}
          aria-label="Закрыть"
          style={{
            position: "absolute",
            top: 6,
            right: 6,
            zIndex: 1,
            background: "rgba(255,255,255,0.75)",
            borderRadius: 4,
          }}
        />

        {/* Левая половина — картинка на всю высоту, та же схема, что в AboutModal. */}
        <div style={{ width: `calc(50% + ${DIVIDER_OFFSET}px)`, height: "100%", flexShrink: 0 }}>
          <img
            src={portrait}
            alt=""
            style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }}
          />
        </div>

        {/* Правая половина — текст сверху, заглушка QR-кода ниже. */}
        <div
          style={{
            width: `calc(50% - ${DIVIDER_OFFSET}px)`,
            height: "100%",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            gap: 24,
            padding: 16,
          }}
        >
          <Text size="sm" fw={600} ta="center">
            Автор благодарен Вам за поддержку!
          </Text>

          {/* TODO: заменить на реальный QR-код (донат-адрес/ссылка), когда
              будет решено, куда именно ведёт поддержка. */}
          <div
            style={{
              width: QR_PLACEHOLDER_SIZE,
              height: QR_PLACEHOLDER_SIZE,
              border: "1px solid var(--mantine-color-gray-5)",
              flexShrink: 0,
            }}
          />

          {/* TODO: заменить на реальный баннер сервиса, когда будет решено,
              куда именно ведёт поддержка (см. TODO у QR-заглушки выше). */}
          <div
            style={{
              width: QR_PLACEHOLDER_SIZE,
              height: SERVICE_BANNER_HEIGHT,
              flexShrink: 0,
            }}
          />
        </div>
      </div>
    </Modal>
  );
}
