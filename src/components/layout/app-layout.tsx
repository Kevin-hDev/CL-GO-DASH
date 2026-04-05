import { type ReactNode } from "react";
import { Sidebar, type TabId } from "./sidebar";

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
    <div className="flex h-screen">
      <Sidebar activeTab={activeTab} onTabChange={onTabChange} />
      <div
        className="bg-[var(--shell)] border-r border-[var(--edge)] flex flex-col overflow-hidden"
        style={{ width: "var(--list-width)", minWidth: "var(--list-width)" }}
      >
        {listContent}
      </div>
      <div className="flex-1 bg-[var(--void)] flex flex-col overflow-hidden">
        {detailContent}
      </div>
    </div>
  );
}
