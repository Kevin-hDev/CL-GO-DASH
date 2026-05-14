import { ModelInstallBtn } from "./model-install-btn";
import { useTranslation } from "react-i18next";
import {
  getForecastHardwareKey,
  getForecastModelSummaryKey,
  type ForecastModelEntry,
} from "../forecast-model-meta";

interface ModelCardProps {
  model: ForecastModelEntry;
  onSelect: () => void;
  onRefresh: () => void;
}

export function ModelCard({ model, onSelect, onRefresh }: ModelCardProps) {
  const { t } = useTranslation();
  const summaryKey = getForecastModelSummaryKey(model.id);
  const summary = t(summaryKey);

  return (
    <div className="fmc-card">
      <button className="fmc-info" onClick={onSelect}>
        <span className="fmc-name">{model.display_name}</span>
        <span className="fmc-summary">{summary === summaryKey ? model.display_name : summary}</span>
        <span className="fmc-meta">
          {model.is_cloud ? (
            <span className="fmc-cloud">
              ☁ {model.provider_configured ? t("forecast.models.cloud") : t("forecast.models.noKeyConfigured")}
            </span>
          ) : (
            <>
              {model.params} · {model.size_mb} MB
              {model.cpu_supported && " · CPU ✓"}
              {model.gpu_supported && " · GPU ✓"}
            </>
          )}
        </span>
        <span className="fmc-ram">
          {t(getForecastHardwareKey(model))}
          {!model.is_cloud ? ` · ${t("forecast.models.ram")}: ~${model.ram_mb} MB` : ""}
        </span>
      </button>
      <div className="fmc-actions">
        {model.is_cloud
          ? <span className="fmc-cloud-badge">☁</span>
          : model.installable
            ? <ModelInstallBtn modelId={model.id} installed={model.installed} onDone={onRefresh} />
            : null}
      </div>
    </div>
  );
}
