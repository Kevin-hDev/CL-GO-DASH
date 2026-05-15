import { useCallback, useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { PanelRightOpen, PanelRightClose } from "lucide-react";
import type { ForecastSection } from "@/hooks/use-forecast-panel";
import { ForecastHeader } from "./forecast-header";
import { ForecastEmpty } from "./forecast-empty";
import {
  buildForecastLayerGroups,
  createInitialLayerState,
  type ForecastLayerState,
} from "./forecast-layer-matrix";
import {
  ForecastViewFilters,
} from "./forecast-view-filters";
import { ExportDropdown } from "./widgets/export-dropdown";
import { ForecastModelSelector } from "./widgets/forecast-model-selector";
import { ForecastConfig, type LaunchConfig } from "./forecast-config";
import { loadForecastDraftFromFile, type ForecastDraftData } from "./forecast-data";
import { useForecastLayerSources } from "./use-forecast-layer-sources";
import { ForecastSectionRouter } from "./forecast-section-router";
import { useCurrentForecastAnalysisName } from "./use-current-forecast-analysis-name";
import { useSelectedForecastModel } from "./use-selected-forecast-model";
import "./forecast-panel.css";

interface ForecastPanelProps {
  activeSection: ForecastSection;
  navOpen: boolean;
  currentAnalysisId: string | null;
  fullscreen: boolean;
  onSectionChange: (section: ForecastSection) => void;
  onToggleNav: () => void;
  onLoadAnalysis: (id: string) => void;
  onFocusAnalysis: (id: string) => void;
  onPanelExtraWidthChange: (width: number) => void;
  onCloseAnalysis: () => void;
  onFullscreenChange: (fs: boolean) => void;
}

export function ForecastPanel({
  activeSection, navOpen, currentAnalysisId, fullscreen,
  onSectionChange, onToggleNav, onLoadAnalysis, onFocusAnalysis, onPanelExtraWidthChange, onCloseAnalysis, onFullscreenChange,
}: ForecastPanelProps) {
  const { t } = useTranslation();
  const hasAnalysis = currentAnalysisId !== null;
  const [draft, setDraft] = useState<ForecastDraftData | null>(null);
  const [launching, setLaunching] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [layers, setLayers] = useState<ForecastLayerState>(createInitialLayerState);
  const [scenarioPickerOpen, setScenarioPickerOpen] = useState(false);
  const { selectedModelId, selectModel, ready: selectedModelReady } = useSelectedForecastModel();
  const currentAnalysisName = useCurrentForecastAnalysisName(currentAnalysisId);
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

  const handleSelectModel = useCallback((modelId: string) => {
    selectModel(modelId);
  }, [selectModel]);

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

  useEffect(() => {
    const nextWidth = activeSection === "scenarios" && scenarioPickerOpen ? 320 : 0;
    onPanelExtraWidthChange(nextWidth);
  }, [activeSection, scenarioPickerOpen, onPanelExtraWidthChange]);

  return (
    <div className="fc-panel">
      <ForecastHeader
        activeSection={activeSection}
        navOpen={navOpen}
        hasAnalysis={hasAnalysis}
        fullscreen={fullscreen}
        contextLabel={activeSection === "scenarios" || activeSection === "comparisons" ? currentAnalysisName : null}
        filterSlot={
          hasAnalysis ? (
            <ForecastViewFilters
              groups={filterGroups}
              layers={layers}
              onChange={setLayers}
            />
          ) : null
        }
        rightSlot={
          activeSection === "scenarios" && hasAnalysis ? (
            <button
              className={`fp-icon-btn ${scenarioPickerOpen ? "fp-icon-btn-active" : ""}`}
              onClick={() => setScenarioPickerOpen((open) => !open)}
              title={t("forecast.scenarios.togglePredictions")}
            >
              {scenarioPickerOpen ? <PanelRightClose size={16} /> : <PanelRightOpen size={16} />}
            </button>
          ) : null
        }
        onToggleNav={onToggleNav}
        onSectionChange={onSectionChange}
        onCloseAnalysis={onCloseAnalysis}
        onFullscreenChange={onFullscreenChange}
      />
      <div className="fc-body">
        {draft ? (
          <ForecastConfig
            draft={draft}
            launching={launching}
            error={error}
            defaultModelId={selectedModelId}
            onModelChange={handleSelectModel}
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
            onFocusAnalysis={onFocusAnalysis}
            onAnalysisChanged={() => void layerSources.refresh()}
            scenarioPickerOpen={scenarioPickerOpen}
          />
        )}
      </div>
      {hasAnalysis && (
        <div className={`fc-footer ${activeSection === "view" ? "" : "fc-footer-end"}`}>
          {activeSection === "view" && (
            <ForecastModelSelector
              selectedModelId={selectedModelId}
              selectionReady={selectedModelReady}
              onSelectModel={handleSelectModel}
            />
          )}
          <ExportDropdown
            analysisId={currentAnalysisId}
            onExport={() => undefined}
          />
        </div>
      )}
    </div>
  );
}
