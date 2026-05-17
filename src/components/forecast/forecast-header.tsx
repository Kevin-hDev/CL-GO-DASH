import type { ReactNode } from "react";
import { Maximize2, Minimize2 } from "lucide-react";
import type { ForecastSection } from "@/hooks/use-forecast-panel";
import { ForecastNav } from "./forecast-nav";

interface ForecastHeaderProps {
  activeSection: ForecastSection;
  navOpen: boolean;
  hasAnalysis: boolean;
  fullscreen: boolean;
  contextLabel?: string | null;
  filterSlot?: ReactNode;
  rightSlot?: ReactNode;
  onToggleNav: () => void;
  onSectionChange: (section: ForecastSection) => void;
  onCloseAnalysis: () => void;
  onFullscreenChange: (fs: boolean) => void;
}

export function ForecastHeader({
  activeSection,
  navOpen,
  hasAnalysis,
  fullscreen,
  contextLabel,
  filterSlot,
  rightSlot,
  onToggleNav,
  onSectionChange,
  onCloseAnalysis,
  onFullscreenChange,
}: ForecastHeaderProps) {
  return (
    <div className="fc-head">
      <div className="fc-head-left">
        {activeSection === "view" && filterSlot}
        {hasAnalysis && (
          <ForecastNav
            open={navOpen}
            activeSection={activeSection}
            onToggle={onToggleNav}
            onSelect={onSectionChange}
          />
        )}
        {contextLabel && <span className="fc-context-label">{contextLabel}</span>}
      </div>
      <div className="fc-head-actions">
        {rightSlot}
        <button
          className="fp-icon-btn"
          onClick={() => onFullscreenChange(!fullscreen)}
          title={fullscreen ? "Réduire" : "Plein écran"}
        >
          {fullscreen ? <Minimize2 size={16} /> : <Maximize2 size={16} />}
        </button>
        {hasAnalysis && (
          <button className="fp-icon-btn" onClick={onCloseAnalysis} title="Fermer">
            <svg width="14" height="14" viewBox="0 0 14 14" fill="none"
              stroke="currentColor" strokeWidth="1.5" strokeLinecap="round">
              <path d="M3 3l8 8M11 3l-8 8" />
            </svg>
          </button>
        )}
      </div>
    </div>
  );
}
