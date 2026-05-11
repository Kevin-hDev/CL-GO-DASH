import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { ModelCard } from "./model-card";
import { ModelSpecs } from "./model-specs";
import "./forecast-models.css";

interface ForecastModel {
  id: string;
  provider_id: string;
  display_name: string;
  params: string;
  size_mb: number;
  ram_mb: number;
  vram_mb: number | null;
  cpu_supported: boolean;
  gpu_supported: boolean;
  horizon_max: number;
  frequencies: string;
  is_cloud: boolean;
  installed: boolean;
  size_on_disk: number;
}

interface Provider {
  id: string;
  display_name: string;
}

export function ForecastModels() {
  const { t } = useTranslation();
  const [models, setModels] = useState<ForecastModel[]>([]);
  const [providers, setProviders] = useState<Provider[]>([]);
  const [selected, setSelected] = useState<ForecastModel | null>(null);

  const refresh = useCallback(() => {
    void invoke<{ models: ForecastModel[]; providers: Provider[] }>("list_forecast_models")
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
    return <ModelSpecs model={selected} onBack={() => setSelected(null)} onRefresh={refresh} />;
  }

  return (
    <div className="fmb-root">
      <div className="fmb-header">
        <span className="fmb-title">{t("forecast.models.title")}</span>
      </div>
      <div className="fmb-list">
        {providers.map((prov) => {
          const provModels = models.filter((m) => m.provider_id === prov.id);
          if (provModels.length === 0) return null;
          return (
            <div key={prov.id} className="fmb-group">
              <span className="fmb-group-label">{prov.display_name}</span>
              {provModels.map((m) => (
                <ModelCard
                  key={m.id}
                  model={m}
                  onSelect={() => setSelected(m)}
                  onRefresh={refresh}
                />
              ))}
            </div>
          );
        })}
      </div>
    </div>
  );
}
