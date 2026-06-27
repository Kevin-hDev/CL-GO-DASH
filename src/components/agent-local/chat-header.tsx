import { useTranslation } from "react-i18next";
import { BookOpen, TerminalSquare } from "lucide-react";
import { DragRegion } from "@/components/layout/drag-region";
import { Tooltip } from "@/components/ui/tooltip";
import { MOD, ALT } from "@/lib/platform";
import type { PanelMode } from "@/hooks/use-forecast-panel";
import { ModeSelector } from "./mode-selector";
import "./chat-header.css";

interface ChatHeaderProps {
  sessionName: string | null;
  sessionId: string | null;
  terminalOpen: boolean;
  previewOpen: boolean;
  showForecastDocs?: boolean;
  panelMode?: PanelMode;
  onToggleTerminal: () => void;
  onTogglePreview: () => void;
  onOpenForecastDocs?: () => void;
  onPanelModeChange?: (mode: PanelMode) => void;
}

export function ChatHeader({
  sessionName,
  sessionId,
  terminalOpen,
  previewOpen,
  showForecastDocs,
  panelMode,
  onToggleTerminal,
  onTogglePreview,
  onOpenForecastDocs,
  onPanelModeChange,
}: ChatHeaderProps) {
  const { t } = useTranslation();
  return (
    <div className="chat-header" role="presentation">
      {sessionName ? (
        <span className="chat-header-title" title={sessionName}>
          {sessionName}
        </span>
      ) : null}
      <DragRegion style={{ flex: 1, minWidth: 0 }} />
      {sessionId && (
        <span className="chat-header-actions">
          {showForecastDocs && onOpenForecastDocs && (
            <Tooltip label={t("forecast.docs.openTooltip")} align="right">
              <button
                className="tab-action-btn"
                onClick={(event) => {
                  event.stopPropagation();
                  onOpenForecastDocs();
                }}
              >
                <BookOpen size={17} />
              </button>
            </Tooltip>
          )}
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
    </div>
  );
}
