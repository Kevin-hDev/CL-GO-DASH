import { cn } from "@/lib/utils";
import { useTranslation } from "react-i18next";
import { UserCircle, ChatsCircle, Gear } from "@/components/ui/icons";
import type { Icon } from "@phosphor-icons/react";
import { ThemedIcon } from "@/components/ui/themed-icon";
import logoSrc from "@/assets/logo.png";
import heartbeatDark from "@/assets/heartbeat.png";
import heartbeatLight from "@/assets/heartbeat-light.png";
import { DragRegion } from "./drag-region";

export type TabId = "heartbeat" | "personality" | "agent-local" | "settings";

interface NavItem {
  id: TabId;
  icon?: Icon;
  imgDark?: string;
  imgLight?: string;
  iconSize?: string;
  i18nKey: string;
}

const NAV_ITEMS: NavItem[] = [
  { id: "agent-local", icon: ChatsCircle, i18nKey: "nav.agentLocal" },
  { id: "heartbeat", imgDark: heartbeatDark, imgLight: heartbeatLight, iconSize: "1.44rem", i18nKey: "nav.heartbeat" },
  { id: "personality", icon: UserCircle, i18nKey: "nav.personality" },
];

const ICON_SIZE = "1.25rem";

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
        style={{ paddingLeft: 8, paddingTop: 8, paddingBottom: 16 }}
      >
        <img src={logoSrc} alt="CL-GO" style={{ width: "2.5rem", height: "2.5rem", borderRadius: 6, flexShrink: 0 }} />
        <span style={{ fontSize: "1.2rem", fontWeight: 700, color: "var(--ink)" }} className="opacity-0 group-hover/sb:opacity-100 transition-opacity duration-150">
          CL-GO
        </span>
      </div>

      {/* Nav items */}
      <div className="flex flex-col flex-1" style={{ gap: 2 }}>
        {NAV_ITEMS.map((item) => (
          <div
            key={item.id}
            onClick={() => onTabChange(item.id)}
            className={cn(
              "relative flex items-center cursor-pointer",
              "whitespace-nowrap",
              "sb-nav-item",
              "transition-all duration-200 ease-out",
            )}
            style={{
              paddingTop: 5,
              paddingBottom: 5,
              marginLeft: 6,
              marginRight: 6,
              borderRadius: "var(--radius-md)",
              background: activeTab === item.id ? "var(--surface-hover)" : undefined,
            }}
          >
            {item.icon ? (
              <item.icon
                size={ICON_SIZE}
                weight={activeTab === item.id ? "fill" : "regular"}
                className={cn(
                  "shrink-0 text-[var(--ink-muted)]",
                  activeTab === item.id && "text-[var(--ink)]",
                )}
              />
            ) : item.imgDark && item.imgLight ? (
              <ThemedIcon
                darkSrc={item.imgDark}
                lightSrc={item.imgLight}
                size={item.iconSize ?? ICON_SIZE}
                style={{
                  flexShrink: 0,
                  opacity: activeTab === item.id ? 1 : 0.5,
                  transition: "opacity 200ms ease-out",
                }}
              />
            ) : null}
            <span
              className={cn(
                "text-sm text-[var(--ink-muted)]",
                "w-0 overflow-hidden opacity-0",
                "group-hover/sb:w-auto group-hover/sb:overflow-visible group-hover/sb:opacity-100",
                "transition-opacity duration-150",
                activeTab === item.id && "text-[var(--ink)] font-medium",
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
          "flex items-center cursor-pointer",
          "whitespace-nowrap",
          "sb-nav-item",
          "transition-all duration-200 ease-out",
        )}
        style={{
          paddingTop: 10,
          paddingBottom: 10,
          marginLeft: 6,
          marginRight: 6,
          marginBottom: 12,
          borderRadius: "var(--radius-md)",
          background: activeTab === "settings" ? "var(--surface-hover)" : undefined,
        }}
      >
        <Gear
          size={ICON_SIZE}
          weight={activeTab === "settings" ? "fill" : "regular"}
          className={cn(
            "shrink-0 text-[var(--ink-faint)]",
            activeTab === "settings" && "text-[var(--ink)]",
          )}
        />
        <span className={cn(
          "text-xs",
          "w-0 overflow-hidden opacity-0",
          "group-hover/sb:w-auto group-hover/sb:overflow-visible group-hover/sb:opacity-100",
          "transition-opacity duration-150",
          activeTab === "settings" ? "text-[var(--ink)]" : "text-[var(--ink-faint)]",
        )}>
          {t("nav.settings")}
        </span>
      </div>
    </nav>
  );
}
