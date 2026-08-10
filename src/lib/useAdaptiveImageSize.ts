import { useEffect, useState } from "react";

// Прямой перенос AlchemistApp::display_size_for из main.rs исходной
// egui-версии: если хотя бы одна сторона исходника больше порога — картинка
// считается "качественной" (Rare Curios и т.п.) и показывается крупнее;
// иначе — это миниатюра с вики (обычно 50×50), и её нарочно показывают
// мельче, чтобы отдельные пиксели не так бросались в глаза при увеличении.
const GOOD_QUALITY_THRESHOLD = 100;
export const GOOD_QUALITY_SIZE = 220;
export const LOW_QUALITY_SIZE = 100;

/**
 * По data-URL картинки определяет, какой размер показа использовать —
 * 220×220 или 100×100. Пока размер исходника ещё не известен (картинка
 * только начала загружаться), возвращает GOOD_QUALITY_SIZE, чтобы не
 * дёргать разметку лишний раз (почти все картинки в базе — "хорошие").
 */
export function useAdaptiveImageSize(dataUrl: string | null): number {
  const [size, setSize] = useState(GOOD_QUALITY_SIZE);

  useEffect(() => {
    if (!dataUrl) {
      setSize(GOOD_QUALITY_SIZE);
      return;
    }
    let cancelled = false;
    const img = new Image();
    img.onload = () => {
      if (cancelled) return;
      const good = img.naturalWidth > GOOD_QUALITY_THRESHOLD || img.naturalHeight > GOOD_QUALITY_THRESHOLD;
      setSize(good ? GOOD_QUALITY_SIZE : LOW_QUALITY_SIZE);
    };
    img.onerror = () => {
      if (!cancelled) setSize(GOOD_QUALITY_SIZE);
    };
    img.src = dataUrl;
    return () => {
      cancelled = true;
    };
  }, [dataUrl]);

  return size;
}
