import { useEffect } from "react";

export function isEditableTarget(target: EventTarget | null): boolean {
  if (target instanceof HTMLInputElement) return true;
  if (target instanceof HTMLTextAreaElement) return true;
  if (target instanceof HTMLElement && target.isContentEditable) return true;
  return false;
}

interface ArrowNavOptions<T> {
  items: T[];
  selectedId: T | null;
  onSelect: (id: T) => void;
  enabled?: boolean;
}

export function useArrowNavigation<T>({
  items,
  selectedId,
  onSelect,
  enabled = true,
}: ArrowNavOptions<T>) {
  useEffect(() => {
    if (!enabled || items.length === 0) return;

    const handler = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      if (isEditableTarget(e.target)) return;
      if (e.key !== "ArrowDown" && e.key !== "ArrowUp") return;

      e.preventDefault();
      const currentIdx = selectedId !== null ? items.indexOf(selectedId) : -1;

      if (e.key === "ArrowDown") {
        const next = currentIdx < items.length - 1 ? currentIdx + 1 : currentIdx;
        if (next !== currentIdx || currentIdx === -1) {
          onSelect(items[Math.max(0, next)]);
        }
      } else {
        const prev = currentIdx > 0 ? currentIdx - 1 : 0;
        if (prev !== currentIdx || currentIdx === -1) {
          onSelect(items[prev]);
        }
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [items, selectedId, onSelect, enabled]);
}
