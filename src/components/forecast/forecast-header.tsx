import type { ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown, Maximize2, Minimize2 } from "lucide-react";
import type { ForecastSection } from "@/hooks/use-forecast-panel";

const SECTION_KEYS: Record<ForecastSection, string> = {
  view: "forecast.nav.mainView",
  scenarios: "forecast.nav.scenarios",
  analysis: "forecast.nav.analysis",
  notes: "forecast.nav.notes",
  history: "forecast.nav.history",
};

interface ForecastHeaderProps {
  activeSection: ForecastSection;
  navOpen: boolean;
  hasAnalysis: boolean;
  fullscreen: boolean;
  filterSlot?: ReactNode;
  onToggleNav: () => void;
  onCloseAnalysis: () => void;
  onFullscreenChange: (fs: boolean) => void;
}

export function ForecastHeader({
  activeSection,
  navOpen,
  hasAnalysis,
  fullscreen,
  filterSlot,
  onToggleNav,
  onCloseAnalysis,
  onFullscreenChange,
}: ForecastHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="fc-head">
      <div className="fc-head-left">
        <span className="fc-title">{t("forecast.title")}</span>
        {activeSection === "view" && filterSlot}
        {hasAnalysis && (
          <button className="fc-nav-trigger" onClick={onToggleNav}>
            <span className="fc-nav-label">{t(SECTION_KEYS[activeSection])}</span>
            <ChevronDown
              size={14}
              style={{
                transform: navOpen ? "rotate(180deg)" : "rotate(0deg)",
                transition: "transform 200ms ease",
              }}
            />
          </button>
        )}
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
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
