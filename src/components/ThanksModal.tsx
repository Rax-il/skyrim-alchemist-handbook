import { useState } from "react";
import { CloseButton, Modal, Select, Text, TextInput } from "@mantine/core";
import portrait from "../assets/thanks-portrait.png";
import ethereumQr from "../assets/donation-ethereum-qr.png";
import ethereumBanner from "../assets/donation-ethereum-metamask.png";

interface Props {
  opened: boolean;
  onClose: () => void;
}

interface DonationOption {
  value: string;
  label: string;
  qrSrc: string;
  address: string;
  bannerSrc: string;
}

// Наполняется поэтапно — пока только один вариант.
const DONATION_OPTIONS: DonationOption[] = [
  {
    value: "ethereum",
    label: "Сеть Ethereum",
    qrSrc: ethereumQr,
    address: "0x0ce4e6492Be3C088bC13E2ba74Ffe0EE61514995",
    bannerSrc: ethereumBanner,
  },
];

const EMPTY_OPTION = "— не выбрано —";

// +20 к THANKS_WIDTH и -10 к DIVIDER_OFFSET вместе расширяют ТОЛЬКО правую
// часть окна на 20px (левая картинка — DIVIDER_OFFSET + WIDTH/2 —
// компенсируется и остаётся прежней ширины), чтобы баннер ниже мог стать
// шире на те же 20px без изменения отступов.
const THANKS_WIDTH = 510;
const THANKS_HEIGHT = 400;
// Пока нет реального QR — просто квадрат-заглушка на его месте (пока
// вариант доната не выбран; после выбора здесь появляется QR выбранного
// варианта).
const QR_PLACEHOLDER_SIZE = 140;
// Заглушка/баннер сервиса — на 20px шире QR-заглушки, высота втрое меньше
// (относительно ширины QR, не своей собственной).
const SERVICE_BANNER_WIDTH = QR_PLACEHOLDER_SIZE + 20;
const SERVICE_BANNER_HEIGHT = QR_PLACEHOLDER_SIZE / 3;
// Смещение разделителя картинка/текст от центра (вправо — положительное).
const DIVIDER_OFFSET = 30;

export function ThanksModal({ opened, onClose }: Props) {
  const [donation, setDonation] = useState<string | null>(null);
  const selected = DONATION_OPTIONS.find((o) => o.value === donation) ?? null;

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

        {/* Правая половина — текст, выбор варианта доната, QR/адрес/баннер
            выбранного варианта. */}
        <div
          style={{
            width: `calc(50% - ${DIVIDER_OFFSET}px)`,
            height: "100%",
            display: "flex",
            flexDirection: "column",
            alignItems: "center",
            justifyContent: "center",
            gap: 12,
            padding: 16,
          }}
        >
          <Text size="sm" fw={600} ta="center">
            Автор благодарен Вам за поддержку!
          </Text>

          <Select
            label="Вариант доната"
            placeholder={EMPTY_OPTION}
            data={DONATION_OPTIONS.map((o) => ({ value: o.value, label: o.label }))}
            value={donation}
            onChange={setDonation}
            clearable
            comboboxProps={{ withinPortal: true }}
            w="100%"
          />

          <div
            style={{
              width: QR_PLACEHOLDER_SIZE,
              height: QR_PLACEHOLDER_SIZE,
              border: selected ? undefined : "1px solid var(--mantine-color-gray-5)",
              flexShrink: 0,
            }}
          >
            {selected && (
              <img
                src={selected.qrSrc}
                alt=""
                style={{ width: "100%", height: "100%", objectFit: "contain", display: "block" }}
              />
            )}
          </div>

          <TextInput value={selected?.address ?? ""} readOnly size="xs" w="100%" />

          <div
            style={{
              width: SERVICE_BANNER_WIDTH,
              height: SERVICE_BANNER_HEIGHT,
              flexShrink: 0,
            }}
          >
            {selected && (
              <img
                src={selected.bannerSrc}
                alt=""
                style={{ width: "100%", height: "100%", objectFit: "contain", display: "block" }}
              />
            )}
          </div>
        </div>
      </div>
    </Modal>
  );
}
