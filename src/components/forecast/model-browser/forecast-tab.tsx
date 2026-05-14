import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { EmptyState } from "@/components/ui/empty-state";
import { ThemedIcon } from "@/components/ui/themed-icon";
import modelsDark from "@/assets/models.png";
import modelsLight from "@/assets/models-light.png";
import {
  listForecastFamilies,
  type ForecastModelEntry,
  type ForecastModelsResponse,
  type ForecastProviderEntry,
} from "../forecast-model-meta";
import { ForecastModels } from "./forecast-models";
import { ModelSpecs } from "./model-specs";
import "@/components/ollama/ollama.css";
import "./forecast-tab.css";

export function ForecastTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const [models, setModels] = useState<ForecastModelEntry[]>([]);
  const [providers, setProviders] = useState<ForecastProviderEntry[]>([]);
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
  }, [refresh]);

  const families = useMemo(() => listForecastFamilies(models), [models]);
  const selectedFamily = useMemo(
    () => families.find((group) => group.id === selectedFamilyId) ?? families[0] ?? null,
    [families, selectedFamilyId],
  );
  const selectedModel = useMemo(
    () => models.find((model) => model.id === selectedModelId) ?? null,
    [models, selectedModelId],
  );

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
      <div className="ollama-subtabs fmt-title-wrap">
        <div className="ollama-subtab active fmt-title-pill">
          <ThemedIcon darkSrc={modelsDark} lightSrc={modelsLight} size="1.2rem" />
          <span className="fmt-title-text">{t("forecast.models.sidebarTitle")}</span>
        </div>
      </div>
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
    </div>
  );

  const detail = selectedModel ? (
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
