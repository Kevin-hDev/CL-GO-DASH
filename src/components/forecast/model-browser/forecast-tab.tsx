import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import { EmptyState } from "@/components/ui/empty-state";
import { ThemedIcon } from "@/components/ui/themed-icon";
import modelfileDark from "@/assets/modelfile.png";
import modelfileLight from "@/assets/modelfile-light.png";
import modelsDark from "@/assets/models.png";
import modelsLight from "@/assets/models-light.png";
import {
  isForecastModelSelectable,
  listForecastFamilies,
  type ForecastModelEntry,
  type ForecastModelsResponse,
  type ForecastProviderEntry,
} from "../forecast-model-meta";
import { ForecastConfigEditor } from "./forecast-config-editor";
import { ForecastConfigList } from "./forecast-config-list";
import { ForecastModels } from "./forecast-models";
import { ModelSpecs } from "./model-specs";
import type { DeepPartial, SettingsNavState } from "@/types/navigation";
import "@/components/ollama/ollama.css";
import "./forecast-tab.css";

interface ForecastTabProps {
  navState: SettingsNavState;
  onNavChange: (partial: DeepPartial<SettingsNavState>) => void;
  onNavReplace: (partial: DeepPartial<SettingsNavState>) => void;
}

export function ForecastTab({ navState, onNavChange, onNavReplace }: ForecastTabProps): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const subTab = navState.forecastSubTab;
  const [models, setModels] = useState<ForecastModelEntry[]>([]);
  const [providers, setProviders] = useState<ForecastProviderEntry[]>([]);
  const selectedConfigModelId = navState.forecastConfigModelId;
  const selectedFamilyId = navState.forecastFamilyId;
  const selectedModelId = navState.forecastModelId;

  const refresh = useCallback(() => {
    void invoke<ForecastModelsResponse>("list_forecast_models")
      .then((result) => {
        setModels(result.models);
        setProviders(result.providers);
      })
      .catch(() => {});
  }, []);

  useEffect(() => {
    refresh();
    const unlistenModels = listen("forecast-models-changed", refresh);
    const unlistenProviders = listen("providers-changed", refresh);
    const unlistenFsProviders = listen("fs:providers-changed", refresh);
    const unlistenFsConfig = listen("fs:config-changed", refresh);
    return () => {
      void unlistenModels.then((fn) => fn());
      void unlistenProviders.then((fn) => fn());
      void unlistenFsProviders.then((fn) => fn());
      void unlistenFsConfig.then((fn) => fn());
    };
  }, [refresh]);

  const families = useMemo(() => listForecastFamilies(models), [models]);
  const configModels = useMemo(() => models.filter(isForecastModelSelectable), [models]);
  const selectedFamily = useMemo(
    () => families.find((group) => group.id === selectedFamilyId) ?? families[0] ?? null,
    [families, selectedFamilyId],
  );
  const selectedConfigModel = useMemo(
    () => configModels.find((model) => model.id === selectedConfigModelId) ?? configModels[0] ?? null,
    [configModels, selectedConfigModelId],
  );
  const selectedModel = useMemo(
    () => models.find((model) => model.id === selectedModelId) ?? null,
    [models, selectedModelId],
  );

  useEffect(() => {
    if (selectedConfigModelId || configModels.length === 0) return;
    void invoke<string | null>("get_selected_forecast_model")
      .then((selected) => {
        const next = configModels.find((model) => model.id === selected)?.id ?? configModels[0]?.id ?? null;
        onNavReplace({ forecastConfigModelId: next });
      })
      .catch(() => onNavReplace({ forecastConfigModelId: configModels[0]?.id ?? null }));
  }, [configModels, selectedConfigModelId, onNavReplace]);

  useEffect(() => {
    if (!selectedFamilyId && families[0]) {
      onNavReplace({ forecastFamilyId: families[0].id });
    }
  }, [families, selectedFamilyId, onNavReplace]);

  useEffect(() => {
    if (selectedFamilyId && !families.some((group) => group.id === selectedFamilyId)) {
      onNavReplace({ forecastFamilyId: families[0]?.id ?? null, forecastModelId: null });
    }
  }, [families, selectedFamilyId, onNavReplace]);

  useEffect(() => {
    if (selectedModelId && !models.some((model) => model.id === selectedModelId)) {
      onNavReplace({ forecastModelId: null });
    }
  }, [models, selectedModelId, onNavReplace]);

  const list = useMemo(() => (
    <div className="fmt-list-root">
      <div className="ollama-subtabs">
        <button
          className={`ollama-subtab ${subTab === "config" ? "active" : ""}`}
          onClick={() => onNavChange({ forecastSubTab: "config" })}
        >
          <ThemedIcon darkSrc={modelfileDark} lightSrc={modelfileLight} size="1.2rem" />
          {t("forecast.modelConfig.sidebarTitle")}
        </button>
        <button
          className={`ollama-subtab ${subTab === "models" ? "active" : ""}`}
          onClick={() => onNavChange({ forecastSubTab: "models" })}
        >
          <ThemedIcon darkSrc={modelsDark} lightSrc={modelsLight} size="1.2rem" />
          {t("forecast.models.sidebarTitle")}
        </button>
      </div>
      {subTab === "config" ? (
        <ForecastConfigList
          models={configModels}
          selectedModelId={selectedConfigModel?.id ?? null}
          onSelect={(id) => onNavChange({ forecastConfigModelId: id })}
        />
      ) : (
        <div className="fmt-family-list">
          {families.map((group) => (
            <button
              key={group.id}
              className={`ollama-model-item fmt-family-item ${selectedFamily?.id === group.id ? "active" : ""}`}
              type="button"
              onClick={() => {
                onNavChange({ forecastFamilyId: group.id, forecastModelId: null });
              }}
            >
              <span className="fmt-family-name">{t(group.titleKey)}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  ), [
    configModels,
    families,
    onNavChange,
    selectedConfigModel?.id,
    selectedFamily?.id,
    subTab,
    t,
  ]);

  const detail = useMemo(() => {
    if (subTab === "config") return <ForecastConfigEditor model={selectedConfigModel} />;
    if (selectedModel) {
      return (
        <ModelSpecs
          model={selectedModel}
          provider={providers.find((item) => item.id === selectedModel.provider_id) ?? null}
          onBack={() => onNavChange({ forecastModelId: null })}
          onRefresh={refresh}
        />
      );
    }
    if (selectedFamily) {
      return (
        <ForecastModels
          group={selectedFamily}
          onSelectModel={(model) => onNavChange({ forecastModelId: model.id })}
          onRefresh={refresh}
        />
      );
    }
    return (
      <div className="fmt-empty">
        <EmptyState message={t("forecast.models.noneAvailable")} />
      </div>
    );
  }, [onNavChange, providers, refresh, selectedConfigModel, selectedFamily, selectedModel, subTab, t]);

  return useMemo(() => ({ list, detail }), [list, detail]);
}
