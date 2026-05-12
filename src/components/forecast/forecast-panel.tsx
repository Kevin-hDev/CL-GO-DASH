import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import type { ForecastSection } from "@/hooks/use-forecast-panel";
import { ForecastHeader } from "./forecast-header";
import { ForecastNav } from "./forecast-nav";
import { ForecastEmpty } from "./forecast-empty";
import {
  buildForecastLayerGroups,
  createInitialLayerState,
  type ForecastLayerState,
} from "./forecast-layer-matrix";
import { ForecastView } from "./sections/forecast-view";
import { ForecastScenarios } from "./sections/forecast-scenarios";
import { ForecastAnalysis } from "./sections/forecast-analysis";
import { ForecastNotes } from "./sections/forecast-notes";
import { ForecastHistory } from "./sections/forecast-history";
import {
  ForecastViewFilters,
} from "./forecast-view-filters";
import { ExportDropdown } from "./widgets/export-dropdown";
import { ForecastConfig, type LaunchConfig } from "./forecast-config";
import { loadForecastDraftFromFile, type ForecastDraftData } from "./forecast-data";
import { useForecastLayerSources } from "./use-forecast-layer-sources";
import "./forecast-panel.css";

interface ForecastPanelProps {
  activeSection: ForecastSection;
  navOpen: boolean;
  currentAnalysisId: string | null;
  fullscreen: boolean;
  onSectionChange: (section: ForecastSection) => void;
  onToggleNav: () => void;
  onLoadAnalysis: (id: string) => void;
  onCloseAnalysis: () => void;
  onFullscreenChange: (fs: boolean) => void;
}

export function ForecastPanel({
  activeSection, navOpen, currentAnalysisId, fullscreen,
  onSectionChange, onToggleNav, onLoadAnalysis, onCloseAnalysis, onFullscreenChange,
}: ForecastPanelProps) {
  const { t } = useTranslation();
  const hasAnalysis = currentAnalysisId !== null;
  const [draft, setDraft] = useState<ForecastDraftData | null>(null);
  const [launching, setLaunching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [layers, setLayers] = useState<ForecastLayerState>(createInitialLayerState);
  const layerSources = useForecastLayerSources(currentAnalysisId, setLayers);
  const filterGroups = buildForecastLayerGroups(
    layerSources.sources,
    t
  );

  const handleImportFile = async (path: string) => {
    setError(null);
    try {
      setDraft(await loadForecastDraftFromFile(path));
    } catch {
      setError(t("forecast.errors.importFailed"));
    }
  };

  const handleLaunch = async (config: LaunchConfig) => {
    if (!draft) return;
    setLaunching(true);
    setError(null);
    try {
      const result = await invoke<{ id: string }>("run_forecast", {
        request: {
          data: draft.dataJson,
          file_path: null,
          target_column: config.targetColumn,
          date_column: config.dateColumn,
          series_column: config.seriesColumn,
          covariate_columns: config.covariates,
          horizon: config.horizon,
          frequency: config.frequency,
          model: config.model,
          confidence_level: config.confidence,
        },
      });
      setDraft(null);
      onLoadAnalysis(result.id);
    } catch {
      setError(t("forecast.errors.launchFailed"));
    } finally {
      setLaunching(false);
    }
  };

  return (
    <div className="fc-panel">
      <ForecastHeader
        activeSection={activeSection}
        navOpen={navOpen}
        hasAnalysis={hasAnalysis}
        fullscreen={fullscreen}
        filterSlot={
          hasAnalysis ? (
            <ForecastViewFilters
              groups={filterGroups}
              layers={layers}
              onChange={setLayers}
            />
          ) : null
        }
        onToggleNav={onToggleNav}
        onCloseAnalysis={onCloseAnalysis}
        onFullscreenChange={onFullscreenChange}
      />
      <ForecastNav open={navOpen} activeSection={activeSection} onSelect={onSectionChange} />
      <div className="fc-body">
        {draft ? (
          <ForecastConfig
            draft={draft}
            launching={launching}
            error={error}
            onLaunch={(config) => void handleLaunch(config)}
            onBack={() => setDraft(null)}
          />
        ) : !hasAnalysis ? (
          <ForecastEmpty
            error={error}
            onLoadAnalysis={onLoadAnalysis}
            onImportFile={(path) => void handleImportFile(path)}
          />
        ) : (
          <ForecastSectionRouter
            section={activeSection}
            analysisId={currentAnalysisId}
            layers={layers}
            onLoadAnalysis={onLoadAnalysis}
            onAnalysisChanged={() => void layerSources.refresh()}
          />
        )}
      </div>
      {hasAnalysis && (
        <div className="fc-footer">
          <ExportDropdown
            analysisId={currentAnalysisId}
            onExport={(format, id) => { console.log("export", format, id); }}
          />
        </div>
      )}
    </div>
  );
}

function ForecastSectionRouter({
  section,
  analysisId,
  layers,
  onLoadAnalysis,
  onAnalysisChanged,
}: {
  section: ForecastSection;
  analysisId: string;
  layers: ForecastLayerState;
  onLoadAnalysis: (id: string) => void;
  onAnalysisChanged: () => void;
}) {
  switch (section) {
    case "view":
      return <ForecastView analysisId={analysisId} layers={layers} />;
    case "scenarios":
      return (
        <ForecastScenarios
          analysisId={analysisId}
          onAnalysisChanged={() => void onAnalysisChanged()}
        />
      );
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
