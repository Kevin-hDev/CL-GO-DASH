import { useState, useEffect, useCallback } from "react";
import { ChevronDown } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import {
  groupForecastModels,
  type ForecastModelEntry,
  type ForecastModelsResponse,
  type ForecastProviderEntry,
} from "../forecast-model-meta";
import { ModelCard } from "./model-card";
import { ModelSpecs } from "./model-specs";
import "./forecast-models.css";

export function ForecastModels() {
  const { t } = useTranslation();
  const [models, setModels] = useState<ForecastModelEntry[]>([]);
  const [providers, setProviders] = useState<ForecastProviderEntry[]>([]);
  const [selected, setSelected] = useState<ForecastModelEntry | null>(null);
  const [openFamilies, setOpenFamilies] = useState<string[]>(["chronos", "timegpt"]);

  const refresh = useCallback(() => {
    void invoke<ForecastModelsResponse>("list_forecast_models")
      .then((result) => {
        setModels(result.models);
        setProviders(result.providers);
        setSelected((current) => {
          if (!current) return null;
          return result.models.find((model) => model.id === current.id) ?? null;
        });
      })
      .catch(() => {});
  }, []);

  useEffect(() => { refresh(); }, [refresh]);

  if (selected) {
    return (
      <ModelSpecs
        model={selected}
        provider={providers.find((item) => item.id === selected.provider_id) ?? null}
        onBack={() => setSelected(null)}
        onRefresh={refresh}
      />
    );
  }

  return (
    <div className="fmb-root">
      <div className="fmb-header">
        <span className="fmb-title">{t("forecast.models.title")}</span>
      </div>
      <div className="fmb-list">
        {groupForecastModels(models).map((group) => {
          const isOpen = openFamilies.includes(group.id);
          return (
            <div key={group.id} className="fmb-group">
              <button
                className="fmb-group-btn"
                type="button"
                onClick={() =>
                  setOpenFamilies((current) =>
                    current.includes(group.id)
                      ? current.filter((item) => item !== group.id)
                      : [...current, group.id]
                  )
                }
              >
                <span className="fmb-group-label">{t(group.titleKey)}</span>
                <span className="fmb-group-count">{group.models.length}</span>
                <ChevronDown
                  size={14}
                  className={`fmb-group-chevron ${isOpen ? "is-open" : ""}`}
                />
              </button>
              <div className={`fmb-group-body ${isOpen ? "is-open" : ""}`}>
                {group.models.map((m) => (
                <ModelCard
                  key={m.id}
                  model={m}
                  onSelect={() => setSelected(m)}
                  onRefresh={refresh}
                />
              ))}
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
}
