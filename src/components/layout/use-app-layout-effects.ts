import { useEffect, useRef, useState, type Dispatch, type SetStateAction } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { IS_MAC } from "@/lib/platform";
import { CHAT_MIN_WIDTH, FILE_PREVIEW_MIN_WIDTH } from "@/hooks/file-preview-storage";
import { FILE_TREE_MIN_WIDTH } from "@/hooks/file-tree-layout";

const SIDEBAR_HIDDEN_OFFSET_FALLBACK = 260;
const SIDEBAR_HIDE_GUARD = 8;

interface ShortcutHandlers {
  onBack: () => void;
  onForward: () => void;
  onNewSession?: () => void;
  toggleSearch: () => void;
  toggleSidebar: () => void;
}

export function shouldAutoHideSidebarForAgentPanels(
  detailWidth: number,
  previewOpen: boolean,
  fileTreeOpen: boolean,
): boolean {
  return previewOpen
    && fileTreeOpen
    && detailWidth < CHAT_MIN_WIDTH + FILE_PREVIEW_MIN_WIDTH + FILE_TREE_MIN_WIDTH;
}

export function sidebarHiddenOffsetFromWidth(width: number): number {
  const safeWidth = Number.isFinite(width) ? Math.max(0, width) : 0;
  return Math.ceil(safeWidth) + SIDEBAR_HIDE_GUARD;
}

export function useWindowFullscreen() {
  const [fullscreen, setFullscreen] = useState(false);

  useEffect(() => {
    let win: ReturnType<typeof getCurrentWindow>;
    try { win = getCurrentWindow(); } catch { return; }

    let active = true;
    let timer: ReturnType<typeof setTimeout>;
    const syncFullscreen = () => {
      void win.isFullscreen().then((value) => {
        if (active) setFullscreen(value);
      }).catch(() => {});
    };

    syncFullscreen();
    const unlisten = win.onResized(() => {
      clearTimeout(timer);
      timer = setTimeout(syncFullscreen, 80);
    });

    return () => {
      active = false;
      clearTimeout(timer);
      cleanupTauriListener(unlisten);
    };
  }, []);

  return fullscreen;
}

export function useAppLayoutShortcuts({
  onBack,
  onForward,
  onNewSession,
  toggleSearch,
  toggleSidebar,
}: ShortcutHandlers) {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = IS_MAC ? e.metaKey : e.ctrlKey;
      if (!mod) return;

      switch (e.code) {
        case "KeyB":
          if (e.altKey) break;
          e.preventDefault();
          toggleSidebar();
          break;
        case "KeyG":
          e.preventDefault();
          toggleSearch();
          break;
        case "ArrowLeft":
          e.preventDefault();
          onBack();
          break;
        case "ArrowRight":
          e.preventDefault();
          onForward();
          break;
        case "KeyN":
          if (e.altKey) {
            e.preventDefault();
            onNewSession?.();
          }
          break;
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [toggleSidebar, toggleSearch, onBack, onForward, onNewSession]);
}

export function useAgentPanelsAutoSidebar(
  sidebarOpen: boolean,
  setSidebarOpen: Dispatch<SetStateAction<boolean>>,
) {
  const sidebarOpenRef = useRef(sidebarOpen);

  useEffect(() => {
    sidebarOpenRef.current = sidebarOpen;
  }, [sidebarOpen]);

  useEffect(() => {
    const detail = document.querySelector(".app-detail-panel");
    if (!(detail instanceof HTMLElement)) return;

    let raf = 0;
    const sync = () => {
      const agentDetail = detail.querySelector(".agent-detail-with-preview");
      const previewOpen = !!agentDetail?.querySelector(".fp-panel.open");
      const fileTreeOpen = !!agentDetail?.querySelector(".ft-panel.open");
      const shouldHide = shouldAutoHideSidebarForAgentPanels(
        detail.getBoundingClientRect().width,
        previewOpen,
        fileTreeOpen,
      );

      if (shouldHide && sidebarOpenRef.current) {
        setSidebarOpen(false);
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
  }, [setSidebarOpen]);
}

export function useSidebarHiddenOffset(sidebarOpen: boolean): number {
  const [offset, setOffset] = useState(SIDEBAR_HIDDEN_OFFSET_FALLBACK);

  useEffect(() => {
    const sidebar = document.querySelector(".app-sidebar-block");
    if (!(sidebar instanceof HTMLElement)) return;

    let raf = 0;
    const measure = () => {
      const next = sidebarHiddenOffsetFromWidth(sidebar.getBoundingClientRect().width);
      setOffset(next);
    };
    const schedule = () => {
      cancelAnimationFrame(raf);
      raf = requestAnimationFrame(measure);
    };

    schedule();
    const resizeObserver = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(schedule);
    resizeObserver?.observe(sidebar);
    for (const child of sidebar.children) resizeObserver?.observe(child);
    window.addEventListener("resize", schedule);

    return () => {
      cancelAnimationFrame(raf);
      resizeObserver?.disconnect();
      window.removeEventListener("resize", schedule);
    };
  }, [sidebarOpen]);

  return offset;
}
