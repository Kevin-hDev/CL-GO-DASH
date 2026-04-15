import { type ReactNode } from "react";
import { Sidebar, type TabId } from "./sidebar";
import { DragRegion } from "./drag-region";

interface AppLayoutProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  listContent: ReactNode;
  detailContent: ReactNode;
}

export function AppLayout({
  activeTab,
  onTabChange,
  listContent,
  detailContent,
}: AppLayoutProps) {
  return (
    <div
      className="flex h-screen"
      style={{
        padding: 6,
        gap: 6,
        background: "var(--void)",
      }}
    >
      <div
        className="flex overflow-hidden"
        style={{
          borderRadius: 10,
          background: "var(--shell)",
          border: "1px solid var(--edge)",
        }}
      >
        <Sidebar activeTab={activeTab} onTabChange={onTabChange} />
        <div
          className="bg-[var(--shell)] flex flex-col overflow-hidden"
          style={{ width: "var(--list-width)", minWidth: "var(--list-width)" }}
        >
          <DragRegion />
          {listContent}
        </div>
      </div>
      <div
        className="flex-1 flex flex-col overflow-hidden"
        style={{
          borderRadius: 10,
          background: "var(--void)",
          border: "1px solid var(--edge)",
        }}
      >
        <DragRegion />
        {detailContent}
      </div>
    </div>
  );
}
