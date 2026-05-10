import { useTranslation } from "react-i18next";
import { TerminalSquare } from "lucide-react";
import { Plus } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { MOD, ALT } from "@/lib/platform";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import { ModeSelector } from "./mode-selector";
import "./tab-bar-actions.css";

interface TabBarActionsProps {
  canAddTab: boolean;
  sessionId: string | null;
  terminalOpen: boolean;
  previewOpen: boolean;
  panelMode?: PanelMode;
  onAdd: () => void;
  onToggleTerminal: () => void;
  onTogglePreview: () => void;
  onPanelModeChange?: (mode: PanelMode) => void;
}

export function TabBarActions({
  canAddTab,
  sessionId,
  terminalOpen,
  previewOpen,
  panelMode,
  onAdd,
  onToggleTerminal,
  onTogglePreview,
  onPanelModeChange,
}: TabBarActionsProps) {
  const { t } = useTranslation();

  return (
    <>
      {canAddTab && (
        <button className="tab-add" onClick={onAdd}>
          <Plus size={14} />
        </button>
      )}
      {sessionId && (
        <span className="tab-actions">
          {previewOpen && panelMode && onPanelModeChange && (
            <ModeSelector mode={panelMode} onChange={onPanelModeChange} />
          )}
          <Tooltip label={`${t("filePreview.togglePanel")} (${ALT}${MOD}B)`} align="right">
            <button
              className={`tab-action-btn ${previewOpen ? "active" : ""}`}
              onClick={(event) => {
                event.stopPropagation();
                onTogglePreview();
              }}
            >
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
                <rect x="1" y="2" width="14" height="12" rx="2" />
                <line x1="10" y1="2" x2="10" y2="14" />
              </svg>
            </button>
          </Tooltip>
          <Tooltip label={`${t("settings.shortcuts.toggleTerminal")} (${MOD}J)`} align="right">
            <button
              className={`tab-action-btn ${terminalOpen ? "active" : ""}`}
              onClick={(event) => {
                event.stopPropagation();
                onToggleTerminal();
              }}
            >
              <TerminalSquare size={18} />
            </button>
          </Tooltip>
        </span>
      )}
    </>
  );
}
