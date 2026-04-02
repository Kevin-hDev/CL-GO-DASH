import { type ReactNode } from "react";
import "./sidebar.css";

export type TabId = "heartbeat" | "history" | "personality";

interface NavItem {
  id: TabId;
  icon: ReactNode;
  label: string;
}

const NAV_ITEMS: NavItem[] = [
  { id: "heartbeat", icon: "⏱", label: "Heartbeat" },
  { id: "history", icon: "📋", label: "Historique" },
  { id: "personality", icon: "👤", label: "Personnalité" },
];

interface SidebarProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
  theme: "dark" | "light";
  onThemeToggle: () => void;
}

export function Sidebar({
  activeTab,
  onTabChange,
  theme,
  onThemeToggle,
}: SidebarProps) {
  return (
    <nav className="sidebar-nav">
      <div className="sidebar-logo">
        <div className="logo-dot" />
        <span className="logo-text">CL-GO</span>
      </div>

      <div className="nav-items">
        {NAV_ITEMS.map((item) => (
          <div
            key={item.id}
            className={`nav-item ${activeTab === item.id ? "active" : ""}`}
            onClick={() => onTabChange(item.id)}
          >
            <div className="nav-icon">{item.icon}</div>
            <span className="nav-label">{item.label}</span>
          </div>
        ))}
      </div>

      <div className="theme-toggle" onClick={onThemeToggle}>
        <div className="nav-icon">{theme === "dark" ? "🌙" : "☀️"}</div>
        <span className="theme-toggle-label">
          {theme === "dark" ? "Dark mode" : "Light mode"}
        </span>
      </div>
    </nav>
  );
}
