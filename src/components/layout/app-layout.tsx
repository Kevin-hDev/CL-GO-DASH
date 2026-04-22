import { useState, useCallback, useEffect, useRef, type ReactNode } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { Sidebar, type TabId } from "./sidebar";
import { DragRegion } from "./drag-region";
import { WindowToolbar } from "./window-toolbar";
import { SearchDialog } from "./search-dialog";
import "./app-layout.css";

interface AppLayoutProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  listContent: ReactNode;
  detailContent: ReactNode;
  onShowWelcome?: () => void;
  onBack: () => void;
  onForward: () => void;
  canGoBack: boolean;
  canGoForward: boolean;
  onSearchSelect: (sessionId: string) => void;
  onNewSession?: () => void;
}

export function AppLayout({
  activeTab, onTabChange,
  listContent, detailContent,
  onShowWelcome,
  onBack, onForward, canGoBack, canGoForward,
  onSearchSelect, onNewSession,
}: AppLayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [searchOpen, setSearchOpen] = useState(false);
  const [fullscreen, setFullscreen] = useState(false);

  useEffect(() => {
    const win = getCurrentWindow();
    let timer: ReturnType<typeof setTimeout>;
    win.isFullscreen().then(setFullscreen).catch(() => {});
    const unlisten = win.onResized(() => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        win.isFullscreen().then(setFullscreen).catch(() => {});
      }, 80);
    });
    return () => { clearTimeout(timer); unlisten.then((fn) => fn()).catch(() => {}); };
  }, []);

  const [listWidth, setListWidth] = useState<number | null>(null);
  const dragging = useRef(false);

  const handleResizeStart = useCallback((e: React.PointerEvent) => {
    e.preventDefault();
    dragging.current = true;
    const startX = e.clientX;
    const listEl = (e.target as HTMLElement).closest(".app-sidebar-block")?.querySelector(".app-list-panel") as HTMLElement;
    if (!listEl) return;
    const startListW = listEl.getBoundingClientRect().width;

    const onMove = (ev: PointerEvent) => {
      const delta = ev.clientX - startX;
      const newList = startListW + delta;
      if (newList >= startListW) setListWidth(newList);
      else setListWidth(null);
    };
    const onUp = () => {
      dragging.current = false;
      window.removeEventListener("pointermove", onMove);
      window.removeEventListener("pointerup", onUp);
    };
    window.addEventListener("pointermove", onMove);
    window.addEventListener("pointerup", onUp);
  }, []);

  const openSearch = useCallback(() => setSearchOpen(true), []);
  const closeSearch = useCallback(() => setSearchOpen(false), []);
  const toggleSidebar = useCallback(() => setSidebarOpen((o) => !o), []);

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      const mod = navigator.userAgent.includes("Mac") ? e.metaKey : e.ctrlKey;
      if (!mod) return;

      switch (e.code) {
        case "KeyB":
          e.preventDefault();
          toggleSidebar();
          break;
        case "KeyG":
          e.preventDefault();
          setSearchOpen((o) => !o);
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
  }, [toggleSidebar, onBack, onForward, onNewSession]);

  return (
    <div className={`app-root ${sidebarOpen ? "" : "sidebar-hidden"} ${fullscreen ? "is-fullscreen" : ""}`}>
      <WindowToolbar
        sidebarOpen={sidebarOpen}
        onToggleSidebar={toggleSidebar}
        onBack={onBack}
        onForward={onForward}
        onNewSession={() => onShowWelcome?.()}
        onSearch={openSearch}
        canGoBack={canGoBack}
        canGoForward={canGoForward}
      />
      <div className={`app-sidebar-block ${sidebarOpen ? "" : "app-sidebar-hidden"}`}>
        <Sidebar activeTab={activeTab} onTabChange={onTabChange} />
        <div
          className="app-list-panel"
          style={listWidth ? { width: listWidth, minWidth: listWidth } : undefined}
        >
          <DragRegion />
          {listContent}
        </div>
        <div className="sidebar-resize-handle" onPointerDown={handleResizeStart} />
      </div>
      <div className="app-detail-panel">
        <DragRegion
          height={22}
          style={{
            position: "absolute",
            top: 0,
            left: 0,
            right: 0,
            zIndex: 1,
          }}
        />
        {detailContent}
      </div>
      <SearchDialog
        open={searchOpen}
        onClose={closeSearch}
        onSelect={onSearchSelect}
      />
    </div>
  );
}
