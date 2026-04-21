import { useTranslation } from "react-i18next";
import { SidebarToggleIcon, ArrowLeftIcon, ArrowRightIcon, SearchIcon } from "./toolbar-icons";
import { ComposeIcon } from "@/components/ui/compose-icon";
import { Tooltip } from "@/components/ui/tooltip";
import "./window-toolbar.css";

const IS_MAC = navigator.userAgent.includes("Mac");
const MOD = IS_MAC ? "⌘" : "Ctrl+";
const ALT = IS_MAC ? "⌥" : "Alt+";

interface WindowToolbarProps {
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
  onBack: () => void;
  onForward: () => void;
  onNewSession: () => void;
  onSearch: () => void;
  canGoBack: boolean;
  canGoForward: boolean;
}

export function WindowToolbar({
  sidebarOpen, onToggleSidebar,
  onBack, onForward, onNewSession, onSearch,
  canGoBack, canGoForward,
}: WindowToolbarProps) {
  const { t } = useTranslation();

  return (
    <div className="window-toolbar">
      <Tooltip label={`${t("settings.shortcuts.toggleSidebar")} (${MOD}B)`}>
        <button className="toolbar-btn" onClick={onToggleSidebar}>
          <SidebarToggleIcon size={16} />
        </button>
      </Tooltip>
      <Tooltip label={`${t("settings.shortcuts.goBack")} (${MOD}◀)`}>
        <button className="toolbar-btn" onClick={onBack} disabled={!canGoBack}>
          <ArrowLeftIcon size={16} />
        </button>
      </Tooltip>
      <Tooltip label={`${t("settings.shortcuts.goForward")} (${MOD}▶)`}>
        <button className="toolbar-btn" onClick={onForward} disabled={!canGoForward}>
          <ArrowRightIcon size={16} />
        </button>
      </Tooltip>
      <Tooltip label={`${t("settings.shortcuts.searchDialog")} (${MOD}G)`}>
        <button className="toolbar-btn" onClick={onSearch}>
          <SearchIcon size={16} />
        </button>
      </Tooltip>
      {!sidebarOpen && (
        <Tooltip label={`${t("settings.shortcuts.newSession")} (${ALT}${MOD}N)`}>
          <button className="toolbar-btn" onClick={onNewSession}>
            <ComposeIcon size={16} />
          </button>
        </Tooltip>
      )}
    </div>
  );
}
