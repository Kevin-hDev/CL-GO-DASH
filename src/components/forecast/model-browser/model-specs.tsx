import { useTranslation } from "react-i18next";
import { ModelInstallBtn } from "./model-install-btn";
import "./model-specs.css";

interface ForecastModel {
  id: string;
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

interface ModelSpecsProps {
  model: ForecastModel;
  onBack: () => void;
  onRefresh: () => void;
}

export function ModelSpecs({ model, onBack, onRefresh }: ModelSpecsProps) {
  const { t } = useTranslation();

  return (
    <div className="fms-root">
      <div className="fms-header">
        <button className="fms-back" onClick={onBack}>← {t("forecast.models.back")}</button>
        <span className="fms-name">{model.display_name}</span>
      </div>
      <div className="fms-body">
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
              <Row label={t("forecast.models.horizonMax")} value={model.horizon_max.toString()} />
              <Row label={t("forecast.models.frequencies")} value={model.frequencies} />
            </div>
          </div>
        )}
        {model.installed && (
          <div className="fms-section">
            <span className="fms-section-title">{t("forecast.models.state")}</span>
            <div className="fms-grid">
              <Row label={t("forecast.models.status")} value={`● ${t("forecast.models.installed")}`} />
              <Row label={t("forecast.models.usedSpace")} value={formatSize(model.size_on_disk)} />
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
