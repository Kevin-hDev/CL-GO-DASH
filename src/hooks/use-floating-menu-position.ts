import { useCallback, useLayoutEffect, useRef, useState, type CSSProperties } from "react";

type FloatingAlign = "left" | "right";

const VIEWPORT_PADDING = 12;
const HIDDEN_STYLE: CSSProperties = {
  position: "fixed",
  top: 0,
  left: 0,
  right: "auto",
  bottom: "auto",
  visibility: "hidden",
  zIndex: 1000,
};

export function useFloatingMenuPosition(open: boolean, align: FloatingAlign = "left", gap = 4) {
  const anchorRef = useRef<HTMLElement | null>(null);
  const floatingRef = useRef<HTMLDivElement | null>(null);
  const [style, setStyle] = useState<CSSProperties>(HIDDEN_STYLE);

  const update = useCallback(() => {
    const anchor = anchorRef.current;
    const floating = floatingRef.current;
    if (!open || !anchor || !floating) return;

    const anchorRect = anchor.getBoundingClientRect();
    const width = floating.offsetWidth;
    const height = floating.offsetHeight;
    const maxLeft = Math.max(VIEWPORT_PADDING, window.innerWidth - width - VIEWPORT_PADDING);
    const rawLeft = align === "right" ? anchorRect.right - width : anchorRect.left;
    const left = Math.min(Math.max(rawLeft, VIEWPORT_PADDING), maxLeft);
    const top = Math.max(VIEWPORT_PADDING, anchorRect.top - height - gap);

    setStyle({
      position: "fixed",
      top,
      left,
      right: "auto",
      bottom: "auto",
      visibility: "visible",
      zIndex: 1000,
    });
  }, [align, gap, open]);

  useLayoutEffect(() => {
    if (!open) return;

    update();
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  }, [open, update]);

  return { anchorRef, floatingRef, floatingStyle: style, updateFloatingPosition: update };
}

export function floatingMenuPortalRoot(): HTMLElement {
  return document.querySelector<HTMLElement>(".app-root") ?? document.body;
}
