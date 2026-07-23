import type { ForecastSection } from "@/hooks/use-forecast-panel";
import type { ForecastLayerState } from "./forecast-layer-matrix";
import { ForecastView } from "./sections/forecast-view";
import { ForecastScenarios } from "./sections/forecast-scenarios";
import { ForecastComparisons } from "./sections/forecast-comparisons";
import { ForecastNotes } from "./sections/forecast-notes";
import { ForecastHistory } from "./sections/forecast-history";

interface ForecastSectionRouterProps {
  section: ForecastSection;
  analysisId: string;
  layers: ForecastLayerState;
  onLoadAnalysis: (id: string) => void;
  onFocusAnalysis: (id: string) => void;
  onAnalysisChanged: () => void;
  scenarioPickerOpen: boolean;
}

export function ForecastSectionRouter({
  section,
  analysisId,
  layers,
  onLoadAnalysis,
  onFocusAnalysis,
  onAnalysisChanged,
  scenarioPickerOpen,
}: ForecastSectionRouterProps) {
  switch (section) {
    case "view":
      return <ForecastView analysisId={analysisId} layers={layers} />;
    case "scenarios":
      return (
        <ForecastScenarios
          analysisId={analysisId}
          onFocusAnalysis={onFocusAnalysis}
          onAnalysisChanged={onAnalysisChanged}
          pickerOpen={scenarioPickerOpen}
        />
      );
    case "comparisons":
      return <ForecastComparisons analysisId={analysisId} />;
    case "notes":
      return <ForecastNotes analysisId={analysisId} />;
    case "history":
      return <ForecastHistory onLoadAnalysis={onLoadAnalysis} />;
    default:
      return null;
  }
}
