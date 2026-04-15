import { cn } from "@/lib/utils";
import { useTranslation } from "react-i18next";
import { Pulse, ClipboardText, UserCircle, ChatCircle, Sliders, Gear } from "@/components/ui/icons";
import type { Icon } from "@phosphor-icons/react";
import logoSrc from "@/assets/logo.png";
import { DragRegion } from "./drag-region";

export type TabId = "heartbeat" | "history" | "personality" | "agent-local" | "ollama" | "settings";

interface NavItem {
  id: TabId;
  icon: Icon;
  i18nKey: string;
}

const NAV_ITEMS: NavItem[] = [
  { id: "heartbeat", icon: Pulse, i18nKey: "nav.heartbeat" },
  { id: "history", icon: ClipboardText, i18nKey: "nav.history" },
  { id: "personality", icon: UserCircle, i18nKey: "nav.personality" },
  { id: "agent-local", icon: ChatCircle, i18nKey: "nav.agentLocal" },
  { id: "ollama", icon: Sliders, i18nKey: "nav.ollama" },
];

const ICON_SIZE = 24;

interface SidebarProps {
  activeTab: TabId;
  onTabChange: (tab: TabId) => void;
}

export function Sidebar({ activeTab, onTabChange }: SidebarProps) {
  const { t } = useTranslation();
  return (
    <nav
      className={cn(
        "group/sb flex flex-col overflow-hidden relative",
        "bg-[var(--shell)]",
        "z-10",
      )}
      style={{
        width: "var(--sidebar-collapsed)",
        minWidth: "var(--sidebar-collapsed)",
        transition: "width 200ms ease-out, min-width 200ms ease-out",
      }}
      onMouseEnter={(e) => {
        e.currentTarget.style.width = "var(--sidebar-expanded)";
        e.currentTarget.style.minWidth = "var(--sidebar-expanded)";
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.width = "var(--sidebar-collapsed)";
        e.currentTarget.style.minWidth = "var(--sidebar-collapsed)";
      }}
    >
      {/* Separator with top fade */}
      <div
        style={{
          position: "absolute",
          top: 0,
          right: 0,
          bottom: 0,
          width: 1,
          background:
            "linear-gradient(to bottom, transparent 0, transparent 50px, var(--edge) 90px, var(--edge) 100%)",
          pointerEvents: "none",
        }}
      />

      {/* Drag region for traffic lights */}
      <DragRegion />

      {/* Logo */}
      <div
        className="flex items-center gap-3 whitespace-nowrap overflow-hidden"
        style={{ paddingLeft: 8, paddingTop: 8, paddingBottom: 24 }}
      >
        <img src={logoSrc} alt="CL-GO" style={{ width: "2.5rem", height: "2.5rem", borderRadius: 6, flexShrink: 0 }} />
        <span style={{ fontSize: "1.2rem", fontWeight: 700, color: "var(--ink)" }} className="opacity-0 group-hover/sb:opacity-100 transition-opacity duration-150">
          CL-GO
        </span>
      </div>

      {/* Nav items */}
      <div className="flex flex-col gap-2 flex-1">
        {NAV_ITEMS.map((item) => (
          <div
            key={item.id}
            onClick={() => onTabChange(item.id)}
            className={cn(
              "relative flex items-center gap-3 cursor-pointer",
              "whitespace-nowrap",
              "transition-all duration-200 ease-out",
            )}
            style={{
              paddingLeft: 10,
              paddingTop: 10,
              paddingBottom: 10,
              marginLeft: 6,
              marginRight: 6,
              borderRadius: "var(--radius-md)",
              background: activeTab === item.id ? "var(--pulse-muted)" : undefined,
            }}
          >
            <item.icon
              size={ICON_SIZE}
              weight={activeTab === item.id ? "fill" : "regular"}
              className={cn(
                "shrink-0 text-[var(--ink-muted)]",
                activeTab === item.id && "text-[var(--pulse)]",
              )}
            />
            <span
              className={cn(
                "text-sm text-[var(--ink-muted)]",
                "opacity-0 group-hover/sb:opacity-100",
                "transition-opacity duration-150",
                activeTab === item.id && "text-[var(--pulse)] font-medium",
              )}
            >
              {t(item.i18nKey)}
            </span>
          </div>
        ))}
      </div>

      {/* Settings */}
      <div
        onClick={() => onTabChange("settings")}
        className={cn(
          "flex items-center gap-3 cursor-pointer",
          "whitespace-nowrap",
          "transition-all duration-200 ease-out",
        )}
        style={{
          paddingLeft: 10,
          paddingTop: 10,
          paddingBottom: 10,
          marginLeft: 6,
          marginRight: 6,
          marginBottom: 12,
          borderRadius: "var(--radius-md)",
          background: activeTab === "settings" ? "var(--pulse-muted)" : undefined,
        }}
      >
        <Gear
          size={ICON_SIZE}
          weight={activeTab === "settings" ? "fill" : "regular"}
          className={cn(
            "shrink-0 text-[var(--ink-faint)]",
            activeTab === "settings" && "text-[var(--pulse)]",
          )}
        />
        <span className={cn(
          "text-xs opacity-0 group-hover/sb:opacity-100 transition-opacity duration-150",
          activeTab === "settings" ? "text-[var(--pulse)]" : "text-[var(--ink-faint)]",
        )}>
          {t("nav.settings")}
        </span>
      </div>
    </nav>
  );
}
