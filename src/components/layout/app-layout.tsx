import { useState, useCallback, useEffect, useRef, type CSSProperties, type ReactNode } from "react";
import { Sidebar, type TabId } from "./sidebar";
import { DragRegion } from "./drag-region";
import { WindowToolbar } from "./window-toolbar";
import { SearchDialog } from "./search-dialog";
import { UpdateNotifications } from "./update-notifications";
import { useUpdateChecker } from "@/hooks/use-update-checker";
import { CHAT_MIN_WIDTH } from "@/hooks/file-preview-storage";
import { IS_MAC } from "@/lib/platform";
import { GpuStatusBadge } from "@/components/agent-local/gpu-status-badge";
import { WindowControls } from "./window-controls";
import { PanelSlotProvider, PanelSlotTarget } from "./panel-slots";
import { useAgentPanelsAutoSidebar, useAppLayoutShortcuts, useSidebarHiddenOffset, useWindowFullscreen } from "./use-app-layout-effects";
import { ModelDownloadBadge } from "./model-download-badge";
import "./app-layout.css";

const GPU_BADGE_OFFSET = 12;
const UPDATES_ANCHOR_MAC = 197;
const UPDATES_ANCHOR_OTHER = 122;

interface AppLayoutProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  onShowWelcome?: () => void;
  onBack: () => void;
  onForward: () => void;
  canGoBack: boolean;
  canGoForward: boolean;
  onSearchSelect: (sessionId: string) => void;
  onNewSession?: () => void;
  children: ReactNode;
}

export function AppLayout({
  activeTab, onTabChange,
  children,
  onShowWelcome,
  onBack, onForward, canGoBack, canGoForward,
  onSearchSelect, onNewSession,
}: AppLayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [manualSidebarReveal, setManualSidebarReveal] = useState(false);
  const [autoSidebarHidden, setAutoSidebarHidden] = useState(false);
  const [searchOpen, setSearchOpen] = useState(false);
  const [updatesOpen, setUpdatesOpen] = useState(false);
  const fullscreen = useWindowFullscreen();
  const updates = useUpdateChecker();
  const sidebarHiddenOffset = useSidebarHiddenOffset(sidebarOpen);

  const [listWidth, setListWidth] = useState<number | null>(null);
  const dragging = useRef(false);

  useEffect(() => {
    const platformClass = IS_MAC ? "os-mac" : "os-other";
    const previousClass = IS_MAC ? "os-other" : "os-mac";
    document.body.classList.add(platformClass);
    document.body.classList.remove(previousClass);

    return () => {
      document.body.classList.remove(platformClass);
    };
  }, []);

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
  const toggleSearch = useCallback(() => setSearchOpen((o) => !o), []);
  const toggleSidebar = useCallback(() => {
    setSidebarOpen((open) => {
      const nextOpen = !open;
      setManualSidebarReveal(nextOpen && autoSidebarHidden);
      if (nextOpen || !autoSidebarHidden) setAutoSidebarHidden(false);
      return nextOpen;
    });
  }, [autoSidebarHidden]);
  const toggleUpdates = useCallback(() => setUpdatesOpen((o) => !o), []);
  const closeUpdates = useCallback(() => setUpdatesOpen(false), []);
  const handleAutoSidebarHide = useCallback(() => {
    setManualSidebarReveal(false);
    setAutoSidebarHidden(true);
    setSidebarOpen(false);
  }, []);

  useAppLayoutShortcuts({ onBack, onForward, onNewSession, toggleSearch, toggleSidebar });
  useAgentPanelsAutoSidebar(
    sidebarOpen,
    manualSidebarReveal,
    handleAutoSidebarHide,
  );
  const compactAgentChat = sidebarOpen && manualSidebarReveal;
  const layoutStyle = {
    "--app-sidebar-hidden-offset": `${sidebarHiddenOffset}px`,
    "--agent-chat-min-width": `${compactAgentChat ? 0 : CHAT_MIN_WIDTH}px`,
  } as CSSProperties;

  return (
    <PanelSlotProvider>
      <div
        className={`app-root ${IS_MAC ? "os-mac" : "os-other"} ${sidebarOpen ? "" : "sidebar-hidden"} ${compactAgentChat ? "agent-chat-compact" : ""} ${fullscreen ? "is-fullscreen" : ""}`}
        style={layoutStyle}
      >
        <WindowControls />
        <WindowToolbar
          sidebarOpen={sidebarOpen}
          onToggleSidebar={toggleSidebar}
          onBack={onBack}
          onForward={onForward}
          onNewSession={() => onShowWelcome?.()}
          onSearch={openSearch}
          onToggleUpdates={toggleUpdates}
          updatesCount={updates.totalCount}
          canGoBack={canGoBack}
          canGoForward={canGoForward}
        />
        <div className={`app-sidebar-block ${sidebarOpen ? "" : "app-sidebar-hidden"}`}>
          <Sidebar activeTab={activeTab} onTabChange={onTabChange} />
          <div
            className="app-list-panel" data-nav-zone="list" tabIndex={-1}
            style={{
              ...(listWidth ? { width: listWidth, minWidth: listWidth } : {}),
              position: "relative",
            }}
          >
            <DragRegion />
            <PanelSlotTarget name="list" />
            <div style={{
              position: "absolute",
              bottom: GPU_BADGE_OFFSET,
              right: GPU_BADGE_OFFSET,
            }}>
              <GpuStatusBadge />
            </div>
          </div>
          <div className="sidebar-resize-handle" onPointerDown={handleResizeStart} />
        </div>
        <div className="app-detail-panel" data-nav-zone="detail" tabIndex={-1}>
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
          <PanelSlotTarget name="detail" />
          <ModelDownloadBadge />
        </div>
        <SearchDialog
          open={searchOpen}
          onClose={closeSearch}
          onSelect={onSearchSelect}
        />
        <UpdateNotifications
          isOpen={updatesOpen}
          onClose={closeUpdates}
          appUpdate={updates.appUpdate}
          ollamaBinaryUpdate={updates.ollamaBinaryUpdate}
          ollamaUpdates={updates.ollamaUpdates}
          pulling={updates.pulling}
          ollamaBinaryUpdating={updates.ollamaBinaryUpdating}
          ollamaBinaryPercent={updates.ollamaBinaryPercent}
          appDownloading={updates.appDownloading}
          appPercent={updates.appPercent}
          onPullModel={(name) => void updates.pullModel(name)}
          onDownloadApp={(url) => void updates.downloadAppUpdate(url)}
          onUpdateOllamaBinary={() => void updates.updateOllamaBinary()}
          anchorLeft={IS_MAC ? UPDATES_ANCHOR_MAC : UPDATES_ANCHOR_OTHER}
        />
        {children}
      </div>
    </PanelSlotProvider>
  );
}
