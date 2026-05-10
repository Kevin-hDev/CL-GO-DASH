import type { ForecastSection } from "@/hooks/use-forecast-panel";
import { ForecastHeader } from "./forecast-header";
import { ForecastNav } from "./forecast-nav";
import { ForecastEmpty } from "./forecast-empty";
import { ForecastView } from "./sections/forecast-view";
import { ForecastScenarios } from "./sections/forecast-scenarios";
import { ForecastAnalysis } from "./sections/forecast-analysis";
import { ForecastNotes } from "./sections/forecast-notes";
import { ForecastHistory } from "./sections/forecast-history";
import "./forecast-panel.css";

interface ForecastPanelProps {
  activeSection: ForecastSection;
  navOpen: boolean;
  currentAnalysisId: string | null;
  onSectionChange: (section: ForecastSection) => void;
  onToggleNav: () => void;
  onLoadAnalysis: (id: string) => void;
  onCloseAnalysis: () => void;
}

export function ForecastPanel({
  activeSection,
  navOpen,
  currentAnalysisId,
  onSectionChange,
  onToggleNav,
  onLoadAnalysis,
  onCloseAnalysis,
}: ForecastPanelProps) {
  const hasAnalysis = currentAnalysisId !== null;

  return (
    <div className="fc-panel">
      <ForecastHeader
        activeSection={activeSection}
        navOpen={navOpen}
        hasAnalysis={hasAnalysis}
        onToggleNav={onToggleNav}
        onCloseAnalysis={onCloseAnalysis}
      />
      <ForecastNav
        open={navOpen}
        activeSection={activeSection}
        onSelect={onSectionChange}
      />
      <div className="fc-body">
        {!hasAnalysis ? (
          <ForecastEmpty onLoadAnalysis={onLoadAnalysis} />
        ) : (
          <ForecastSectionRouter
            section={activeSection}
            analysisId={currentAnalysisId}
            onLoadAnalysis={onLoadAnalysis}
          />
        )}
      </div>
    </div>
  );
}

function ForecastSectionRouter({ section, analysisId, onLoadAnalysis }: {
  section: ForecastSection;
  analysisId: string;
  onLoadAnalysis: (id: string) => void;
}) {
  switch (section) {
    case "view":
      return <ForecastView analysisId={analysisId} />;
    case "scenarios":
      return <ForecastScenarios analysisId={analysisId} />;
    case "analysis":
      return <ForecastAnalysis analysisId={analysisId} />;
    case "notes":
      return <ForecastNotes analysisId={analysisId} />;
    case "history":
      return <ForecastHistory onLoadAnalysis={onLoadAnalysis} />;
    default:
      return null;
  }
}
