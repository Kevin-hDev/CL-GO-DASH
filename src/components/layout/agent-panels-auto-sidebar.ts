import { useEffect, useLayoutEffect, useRef } from "react";
import { CHAT_MIN_WIDTH, FILE_PREVIEW_MIN_WIDTH } from "@/hooks/file-preview-storage";
import { FILE_TREE_MIN_WIDTH } from "@/hooks/file-tree-layout";

export function shouldAutoHideSidebarForAgentPanels(
  detailWidth: number,
  previewOpen: boolean,
  fileTreeOpen: boolean,
): boolean {
  return previewOpen
    && fileTreeOpen
    && detailWidth < CHAT_MIN_WIDTH + FILE_PREVIEW_MIN_WIDTH + FILE_TREE_MIN_WIDTH;
}

export function projectedDetailWidthWithSidebarOpen(
  detailWidth: number,
  sidebarWidth: number,
  sidebarOpen: boolean,
): number {
  const safeDetailWidth = Number.isFinite(detailWidth) ? Math.max(0, detailWidth) : 0;
  if (sidebarOpen) return safeDetailWidth;
  const safeSidebarWidth = Number.isFinite(sidebarWidth) ? Math.max(0, sidebarWidth) : 0;
  return Math.max(0, safeDetailWidth - safeSidebarWidth);
}

export function useAgentPanelsAutoSidebar(
  sidebarOpen: boolean,
  manualReveal: boolean,
  onAutoHide: () => void,
  onTightChange?: (tight: boolean) => void,
) {
  const sidebarOpenRef = useRef(sidebarOpen);
  const manualRevealRef = useRef(manualReveal);

  useLayoutEffect(() => {
    sidebarOpenRef.current = sidebarOpen;
    manualRevealRef.current = manualReveal;
  }, [sidebarOpen, manualReveal]);

  useEffect(() => {
    const detail = document.querySelector(".app-detail-panel");
    const sidebar = document.querySelector(".app-sidebar-block");
    if (!(detail instanceof HTMLElement)) return;

    let raf = 0;
    const sync = () => {
      const agentDetail = detail.querySelector(".agent-detail-with-preview");
      const previewOpen = !!agentDetail?.querySelector(".fp-panel.open");
      const fileTreeOpen = !!agentDetail?.querySelector(".ft-panel.open");
      const sidebarWidth = sidebar instanceof HTMLElement ? sidebar.getBoundingClientRect().width : 0;
      const projectedDetailWidth = projectedDetailWidthWithSidebarOpen(
        detail.getBoundingClientRect().width,
        sidebarWidth,
        sidebarOpenRef.current,
      );
      const shouldHide = shouldAutoHideSidebarForAgentPanels(
        projectedDetailWidth,
        previewOpen,
        fileTreeOpen,
      );

      onTightChange?.(shouldHide);
      if (shouldHide && sidebarOpenRef.current && !manualRevealRef.current) {
        onAutoHide();
      }
    };

    const schedule = () => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(sync);
    };

    schedule();
    const resizeObserver = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(schedule);
    resizeObserver?.observe(detail);
    const mutationObserver = new MutationObserver(schedule);
    mutationObserver.observe(detail, { attributes: true, attributeFilter: ["class"], subtree: true });
    window.addEventListener("resize", schedule);

    return () => {
      cancelAnimationFrame(raf);
      resizeObserver?.disconnect();
      mutationObserver.disconnect();
      window.removeEventListener("resize", schedule);
    };
  }, [onAutoHide, onTightChange]);
}
