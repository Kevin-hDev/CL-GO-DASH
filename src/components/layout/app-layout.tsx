import { useState, type ReactNode } from "react";
import { Sidebar, type TabId } from "./sidebar";
import { DragRegion } from "./drag-region";
import { WindowToolbar } from "./window-toolbar";
import "./app-layout.css";

interface AppLayoutProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  listContent: ReactNode;
  detailContent: ReactNode;
  hideDetailDrag?: boolean;
  onShowWelcome?: () => void;
}

export function AppLayout({
  activeTab, onTabChange,
  listContent, detailContent,
  hideDetailDrag = false, onShowWelcome,
}: AppLayoutProps) {
  const [sidebarOpen, setSidebarOpen] = useState(true);

  return (
    <div className={`app-root ${sidebarOpen ? "" : "sidebar-hidden"}`}>
      <WindowToolbar
        sidebarOpen={sidebarOpen}
        onToggleSidebar={() => setSidebarOpen((o) => !o)}
        onBack={() => {}}
        onForward={() => {}}
        onNewSession={() => onShowWelcome?.()}
        canGoBack={false}
        canGoForward={false}
      />
      <div className={`app-sidebar-block ${sidebarOpen ? "" : "app-sidebar-hidden"}`}>
        <Sidebar activeTab={activeTab} onTabChange={onTabChange} />
        <div className="app-list-panel">
          <DragRegion />
          {listContent}
        </div>
      </div>
      <div className="app-detail-panel">
        {detailContent}
      </div>
    </div>
  );
}
