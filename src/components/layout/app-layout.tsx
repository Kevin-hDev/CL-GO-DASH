import { type ReactNode } from "react";
import { Sidebar, type TabId } from "./sidebar";
import { ListPanel } from "./list-panel";
import { DetailPanel } from "./detail-panel";
import "./app-layout.css";

interface AppLayoutProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  theme: "dark" | "light";
  onThemeToggle: () => void;
  listContent: ReactNode;
  detailContent: ReactNode;
}

export function AppLayout({
  activeTab,
  onTabChange,
  theme,
  onThemeToggle,
  listContent,
  detailContent,
}: AppLayoutProps) {
  return (
    <div className="app-layout">
      <Sidebar
        activeTab={activeTab}
        onTabChange={onTabChange}
        theme={theme}
        onThemeToggle={onThemeToggle}
      />
      <ListPanel>{listContent}</ListPanel>
      <DetailPanel>{detailContent}</DetailPanel>
    </div>
  );
}
