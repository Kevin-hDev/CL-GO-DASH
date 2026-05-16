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
import "@/components/ollama/ollama.css";
import "./forecast-tab.css";

type ForecastSettingsSubTab = "config" | "models";

export function ForecastTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const [subTab, setSubTab] = useState<ForecastSettingsSubTab>("config");
  const [models, setModels] = useState<ForecastModelEntry[]>([]);
  const [providers, setProviders] = useState<ForecastProviderEntry[]>([]);
  const [selectedConfigModelId, setSelectedConfigModelId] = useState<string | null>(null);
  const [selectedFamilyId, setSelectedFamilyId] = useState<string | null>(null);
  const [selectedModelId, setSelectedModelId] = useState<string | null>(null);

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
        setSelectedConfigModelId(next);
      })
      .catch(() => setSelectedConfigModelId(configModels[0]?.id ?? null));
  }, [configModels, selectedConfigModelId]);

  useEffect(() => {
    if (!selectedFamilyId && families[0]) {
      setSelectedFamilyId(families[0].id);
    }
  }, [families, selectedFamilyId]);

  useEffect(() => {
    if (selectedFamilyId && !families.some((group) => group.id === selectedFamilyId)) {
      setSelectedFamilyId(families[0]?.id ?? null);
      setSelectedModelId(null);
    }
  }, [families, selectedFamilyId]);

  useEffect(() => {
    if (selectedModelId && !models.some((model) => model.id === selectedModelId)) {
      setSelectedModelId(null);
    }
  }, [models, selectedModelId]);

  const list = (
    <div className="fmt-list-root">
      <div className="ollama-subtabs">
        <button
          className={`ollama-subtab ${subTab === "config" ? "active" : ""}`}
          onClick={() => setSubTab("config")}
        >
          <ThemedIcon darkSrc={modelfileDark} lightSrc={modelfileLight} size="1.2rem" />
          {t("forecast.modelConfig.sidebarTitle")}
        </button>
        <button
          className={`ollama-subtab ${subTab === "models" ? "active" : ""}`}
          onClick={() => setSubTab("models")}
        >
          <ThemedIcon darkSrc={modelsDark} lightSrc={modelsLight} size="1.2rem" />
          {t("forecast.models.sidebarTitle")}
        </button>
      </div>
      {subTab === "config" ? (
        <ForecastConfigList
          models={configModels}
          selectedModelId={selectedConfigModel?.id ?? null}
          onSelect={setSelectedConfigModelId}
        />
      ) : (
        <div className="fmt-family-list">
          {families.map((group) => (
            <button
              key={group.id}
              className={`ollama-model-item fmt-family-item ${selectedFamily?.id === group.id ? "active" : ""}`}
              type="button"
              onClick={() => {
                setSelectedFamilyId(group.id);
                setSelectedModelId(null);
              }}
            >
              <span className="fmt-family-name">{t(group.titleKey)}</span>
            </button>
          ))}
        </div>
      )}
    </div>
  );

  const detail = subTab === "config" ? (
    <ForecastConfigEditor model={selectedConfigModel} />
  ) : selectedModel ? (
    <ModelSpecs
      model={selectedModel}
      provider={providers.find((item) => item.id === selectedModel.provider_id) ?? null}
      onBack={() => setSelectedModelId(null)}
      onRefresh={refresh}
    />
  ) : selectedFamily ? (
    <ForecastModels
      group={selectedFamily}
      onSelectModel={(model) => setSelectedModelId(model.id)}
      onRefresh={refresh}
    />
  ) : (
    <div className="fmt-empty">
      <EmptyState message={t("forecast.models.noneAvailable")} />
    </div>
  );

  return { list, detail };
}
