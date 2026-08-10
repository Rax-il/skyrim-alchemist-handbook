import { useCallback, useRef } from "react";

/**
 * Универсальный хук для перетаскивания разделителя мышью.
 * onDelta получает разницу в пикселях с прошлого события move.
 */
export function useDragHandle(onDelta: (deltaX: number, deltaY: number) => void) {
  const last = useRef<{ x: number; y: number } | null>(null);

  const onMouseMove = useCallback(
    (e: MouseEvent) => {
      if (!last.current) return;
      const dx = e.clientX - last.current.x;
      const dy = e.clientY - last.current.y;
      last.current = { x: e.clientX, y: e.clientY };
      onDelta(dx, dy);
    },
    [onDelta],
  );

  const onMouseUp = useCallback(() => {
    last.current = null;
    window.removeEventListener("mousemove", onMouseMove);
    window.removeEventListener("mouseup", onMouseUp);
  }, [onMouseMove]);

  const onMouseDown = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      last.current = { x: e.clientX, y: e.clientY };
      window.addEventListener("mousemove", onMouseMove);
      window.addEventListener("mouseup", onMouseUp);
    },
    [onMouseMove, onMouseUp],
  );

  return onMouseDown;
}
