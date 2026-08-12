import { useEffect, useState } from "react";
import { Button, CloseButton, Modal, Text } from "@mantine/core";
import { getVersion } from "@tauri-apps/api/app";
import { useTranslation } from "react-i18next";
import portrait from "../assets/about-portrait.png";

interface Props {
  opened: boolean;
  onClose: () => void;
  onDonateClick: () => void;
}

// Общий размер окна — подгоняем по месту, отсюда же берётся высота
// содержимого (ширина всего окна задаётся через size у Modal).
const ABOUT_WIDTH = 500;
// Высота текстового блока не менялась при увеличении окна — весь прирост
// (+30) ушёл в новую нижнюю полосу с кнопкой, чтобы шрифт/пропорции текста
// остались как были.
const TEXT_AREA_HEIGHT = 200;
const BUTTON_AREA_HEIGHT = 30;
const ABOUT_HEIGHT = TEXT_AREA_HEIGHT + BUTTON_AREA_HEIGHT;

const AUTHOR = "Rax_il";

export function AboutModal({ opened, onClose, onDonateClick }: Props) {
  const { t } = useTranslation();
  // Версия приложения — берём динамически из tauri.conf.json через API,
  // чтобы не дублировать и не забывать поправить строку при бампе версии.
  const [version, setVersion] = useState<string | null>(null);
  useEffect(() => {
    if (!opened) return;
    getVersion()
      .then(setVersion)
      .catch(() => setVersion(null));
  }, [opened]);

  return (
    <Modal
      opened={opened}
      onClose={onClose}
      withCloseButton={false}
      padding={0}
      size={ABOUT_WIDTH}
      centered
      styles={{
        content: { overflow: "hidden" },
        body: { padding: 0 },
      }}
    >
      <div style={{ position: "relative", width: "100%", height: ABOUT_HEIGHT, display: "flex" }}>
        <CloseButton
          onClick={onClose}
          aria-label={t("common.close")}
          style={{
            position: "absolute",
            top: 6,
            right: 6,
            zIndex: 1,
            background: "rgba(255,255,255,0.75)",
            borderRadius: 4,
          }}
        />

        {/* Левая половина — статичное изображение, зашитое в бандл на этапе
            сборки (import из src/assets, никак не меняется во время работы). */}
        <div style={{ width: "50%", height: "100%", flexShrink: 0 }}>
          <img
            src={portrait}
            alt=""
            style={{ width: "100%", height: "100%", objectFit: "cover", display: "block" }}
          />
        </div>

        {/* Правая половина — делится по горизонтали на текстовый блок сверху
            и кнопку снизу. */}
        <div style={{ width: "50%", height: "100%", display: "flex", flexDirection: "column" }}>
          <div
            style={{
              height: TEXT_AREA_HEIGHT,
              flexShrink: 0,
              display: "flex",
              flexDirection: "column",
              alignItems: "center",
              justifyContent: "center",
              textAlign: "center",
              padding: 12,
              gap: 8,
            }}
          >
            <Text fw={600} size="sm">
              {t("appTitle")}
            </Text>
            <Text size="xs" c="dimmed">
              {t("aboutModal.versionLabel", { version: version ?? "…" })}
            </Text>
            <Text size="xs">{t("aboutModal.authorLabel", { author: AUTHOR })}</Text>
            <Text size="xs" style={{ marginTop: 8 }}>
              {t("aboutModal.summary")}
            </Text>
          </div>

          <div
            style={{
              height: BUTTON_AREA_HEIGHT,
              flexShrink: 0,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
            }}
          >
            <Button variant="light" size="compact-xs" onClick={onDonateClick}>
              {t("aboutModal.donateButton")}
            </Button>
          </div>
        </div>
      </div>
    </Modal>
  );
}
