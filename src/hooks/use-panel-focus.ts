import { useState, useEffect, useCallback } from "react";
import { isEditableTarget } from "./use-arrow-navigation";

export type NavPanel = "sidebar" | "list";

export function usePanelFocus() {
  const [focusedPanel, setFocusedPanel] = useState<NavPanel>("list");

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      if (isEditableTarget(e.target)) return;

      if (e.key === "ArrowRight" && focusedPanel === "sidebar") {
        e.preventDefault();
        setFocusedPanel("list");
      } else if (e.key === "ArrowLeft" && focusedPanel === "list") {
        e.preventDefault();
        setFocusedPanel("sidebar");
      }
    };

    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [focusedPanel]);

  const resetToList = useCallback(() => setFocusedPanel("list"), []);

  return { focusedPanel, setFocusedPanel, resetToList };
}
