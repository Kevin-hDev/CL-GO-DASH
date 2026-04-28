import { useTranslation } from "react-i18next";
import { SidebarToggleIcon, ArrowLeftIcon, ArrowRightIcon, SearchIcon } from "./toolbar-icons";
import { ComposeIcon } from "@/components/ui/compose-icon";
import { Tooltip } from "@/components/ui/tooltip";
import { IS_MAC, MOD, ALT } from "@/lib/platform";
import updateIcon from "@/assets/update.png";
import "./window-toolbar.css";

interface WindowToolbarProps {
  sidebarOpen: boolean;
  onToggleSidebar: () => void;
  onBack: () => void;
  onForward: () => void;
  onNewSession: () => void;
  onSearch: () => void;
  onToggleUpdates: () => void;
  updatesCount: number;
  canGoBack: boolean;
  canGoForward: boolean;
  previewOpen: boolean;
  previewCount: number;
  onTogglePreview: () => void;
}

export function WindowToolbar({
  sidebarOpen, onToggleSidebar,
  onBack, onForward, onNewSession, onSearch,
  onToggleUpdates, updatesCount,
  canGoBack, canGoForward,
  previewOpen, previewCount, onTogglePreview,
}: WindowToolbarProps) {
  const { t } = useTranslation();

  return (
    <div className={`window-toolbar${IS_MAC ? " is-mac" : ""}`}>
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
      <Tooltip label={`Aperçu fichiers (${ALT}${MOD}B)`}>
        <button
          className={`toolbar-btn${previewOpen ? " toolbar-btn-active" : ""}`}
          onClick={onTogglePreview}
        >
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
            <rect x="1" y="2" width="14" height="12" rx="2" />
            <line x1="10" y1="2" x2="10" y2="14" />
          </svg>
          {previewCount > 0 && (
            <span className="toolbar-badge">{previewCount}</span>
          )}
        </button>
      </Tooltip>
      {sidebarOpen && updatesCount > 0 && (
        <Tooltip label={t("updates.tooltip")}>
          <button className="toolbar-btn toolbar-btn-update" onClick={onToggleUpdates}>
            <img src={updateIcon} alt="" style={{ width: 16, height: 16 }} />
          </button>
        </Tooltip>
      )}
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
