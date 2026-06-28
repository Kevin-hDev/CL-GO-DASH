import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { IS_MAC } from "@/lib/platform";

interface ShortcutHandlers {
  onBack: () => void;
  onForward: () => void;
  onNewSession?: () => void;
  toggleSearch: () => void;
  toggleSidebar: () => void;
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
