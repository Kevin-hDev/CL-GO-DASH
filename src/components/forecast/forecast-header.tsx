import { ChevronDown } from "lucide-react";
import type { ForecastSection } from "@/hooks/use-forecast-panel";

const SECTION_LABELS: Record<ForecastSection, string> = {
  view: "Vue principale",
  scenarios: "Scénarios",
  analysis: "Analyse",
  notes: "Notes",
  history: "Historique",
};

interface ForecastHeaderProps {
  activeSection: ForecastSection;
  navOpen: boolean;
  hasAnalysis: boolean;
  onToggleNav: () => void;
  onCloseAnalysis: () => void;
}

export function ForecastHeader({
  activeSection,
  navOpen,
  hasAnalysis,
  onToggleNav,
  onCloseAnalysis,
}: ForecastHeaderProps) {
  return (
    <div className="fc-head">
      <div className="fc-head-left">
        <span className="fc-title">Forecast</span>
        {hasAnalysis && (
          <button className="fc-nav-trigger" onClick={onToggleNav}>
            <span className="fc-nav-label">{SECTION_LABELS[activeSection]}</span>
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
      {hasAnalysis && (
        <button className="fp-icon-btn" onClick={onCloseAnalysis} title="Fermer l'analyse">
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" stroke="currentColor"
            strokeWidth="1.5" strokeLinecap="round">
            <path d="M3 3l8 8M11 3l-8 8" />
          </svg>
        </button>
      )}
    </div>
  );
}
