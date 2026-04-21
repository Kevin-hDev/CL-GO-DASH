import { useState, useCallback, useEffect, type ReactNode } from "react";
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
    <div className={`app-root ${sidebarOpen ? "" : "sidebar-hidden"}`}>
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
        <div className="app-list-panel">
          <DragRegion />
          {listContent}
        </div>
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
