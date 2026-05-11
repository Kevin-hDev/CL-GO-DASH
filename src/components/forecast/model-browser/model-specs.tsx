import { useTranslation } from "react-i18next";
import { ModelInstallBtn } from "./model-install-btn";
import {
  getForecastHardwareKey,
  getForecastModelSummaryKey,
  type ForecastModelEntry,
  type ForecastProviderEntry,
} from "../forecast-model-meta";
import "./model-specs.css";

interface ModelSpecsProps {
  model: ForecastModelEntry;
  provider: ForecastProviderEntry | null;
  onBack: () => void;
  onRefresh: () => void;
}

export function ModelSpecs({ model, provider, onBack, onRefresh }: ModelSpecsProps) {
  const { t } = useTranslation();

  return (
    <div className="fms-root">
      <div className="fms-header">
        <button className="fms-back" onClick={onBack}>← {t("forecast.models.back")}</button>
        <span className="fms-name">{model.display_name}</span>
      </div>
      <div className="fms-body">
        <div className="fms-section">
          <span className="fms-section-title">{t("forecast.models.summary")}</span>
          <p className="fms-summary">{t(getForecastModelSummaryKey(model.id))}</p>
          <div className="fms-tags">
            <span className="fms-tag">{t(getForecastHardwareKey(model))}</span>
            {model.capabilities?.past_covariates && (
              <span className="fms-tag">{t("forecast.models.capabilities.context")}</span>
            )}
            {model.capabilities?.multivariate && (
              <span className="fms-tag">{t("forecast.models.capabilities.multivariate")}</span>
            )}
            {model.capabilities?.probabilistic && (
              <span className="fms-tag">{t("forecast.models.capabilities.probabilistic")}</span>
            )}
          </div>
        </div>
        {!model.is_cloud && (
          <div className="fms-section">
            <span className="fms-section-title">{t("forecast.models.specs")}</span>
            <div className="fms-grid">
              <Row label={t("forecast.models.parameters")} value={model.params} />
              <Row label={t("forecast.models.diskSize")} value={`${model.size_mb} MB`} />
              <Row label={t("forecast.models.ramCpu")} value={`~${model.ram_mb} MB`} />
              {model.vram_mb && <Row label={t("forecast.models.vramGpu")} value={`~${model.vram_mb} MB`} />}
              <Row label="CPU" value={model.cpu_supported ? t("forecast.models.supported") : t("forecast.models.unsupported")} />
              <Row label="GPU" value={model.gpu_supported ? t("forecast.models.supported") : t("forecast.models.unsupported")} />
              <Row label={t("forecast.models.capabilities.context")} value={model.capabilities?.past_covariates ? t("forecast.models.supported") : t("forecast.models.unsupported")} />
              <Row label={t("forecast.models.capabilities.futureContext")} value={model.capabilities?.future_covariates ? t("forecast.models.supported") : t("forecast.models.unsupported")} />
              <Row label={t("forecast.models.capabilities.multivariate")} value={model.capabilities?.multivariate ? t("forecast.models.supported") : t("forecast.models.unsupported")} />
              <Row label={t("forecast.models.capabilities.probabilistic")} value={model.capabilities?.probabilistic ? t("forecast.models.supported") : t("forecast.models.unsupported")} />
              <Row label={t("forecast.models.capabilities.backtesting")} value={model.capabilities?.backtesting_ready ? t("forecast.models.supported") : t("forecast.models.unsupported")} />
              <Row label={t("forecast.models.horizonMax")} value={model.horizon_max.toString()} />
              <Row label={t("forecast.models.frequencies")} value={model.frequencies} />
            </div>
          </div>
        )}
        {model.is_cloud && provider && (
          <div className="fms-section">
            <span className="fms-section-title">{t("forecast.models.state")}</span>
            <div className="fms-grid">
              <Row label={t("forecast.models.status")} value={provider.configured ? t("forecast.models.cloud") : t("forecast.models.noKeyConfigured")} />
              <Row label={t("forecast.models.capabilities.context")} value={model.capabilities?.past_covariates ? t("forecast.models.supported") : t("forecast.models.unsupported")} />
              <Row label={t("forecast.models.capabilities.futureContext")} value={model.capabilities?.future_covariates ? t("forecast.models.supported") : t("forecast.models.unsupported")} />
              <Row label={t("forecast.models.capabilities.multivariate")} value={model.capabilities?.multivariate ? t("forecast.models.supported") : t("forecast.models.unsupported")} />
              <Row label={t("forecast.models.capabilities.probabilistic")} value={model.capabilities?.probabilistic ? t("forecast.models.supported") : t("forecast.models.unsupported")} />
              <Row label={t("forecast.models.capabilities.backtesting")} value={model.capabilities?.backtesting_ready ? t("forecast.models.supported") : t("forecast.models.unsupported")} />
            </div>
          </div>
        )}
        {model.installed && (
          <div className="fms-section">
            <span className="fms-section-title">{t("forecast.models.state")}</span>
            <div className="fms-grid">
              <Row label={t("forecast.models.status")} value={`● ${t("forecast.models.installed")}`} />
              <Row label={t("forecast.models.usedSpace")} value={formatSize(model.size_on_disk ?? 0)} />
            </div>
          </div>
        )}
      </div>
      <div className="fms-footer">
        {!model.is_cloud && (
          <ModelInstallBtn modelId={model.id} installed={model.installed} onDone={onRefresh} />
        )}
      </div>
    </div>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="fms-row">
      <span className="fms-label">{label}</span>
      <span className="fms-value">{value}</span>
    </div>
  );
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1073741824) return `${(bytes / 1048576).toFixed(1)} MB`;
  return `${(bytes / 1073741824).toFixed(2)} GB`;
}
