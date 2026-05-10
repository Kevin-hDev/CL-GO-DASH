import { invoke } from "@tauri-apps/api/core";
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
  const handleUninstall = async () => {
    await invoke("uninstall_forecast_model", { name: model.id });
    onRefresh();
    onBack();
  };

  return (
    <div className="fms-root">
      <div className="fms-header">
        <button className="fms-back" onClick={onBack}>← Retour</button>
        <span className="fms-name">{model.display_name}</span>
      </div>
      <div className="fms-body">
        {!model.is_cloud && (
          <div className="fms-section">
            <span className="fms-section-title">Spécifications</span>
            <div className="fms-grid">
              <Row label="Paramètres" value={model.params} />
              <Row label="Taille disque" value={`${model.size_mb} MB`} />
              <Row label="RAM (CPU)" value={`~${model.ram_mb} MB`} />
              {model.vram_mb && <Row label="VRAM (GPU)" value={`~${model.vram_mb} MB`} />}
              <Row label="CPU" value={model.cpu_supported ? "✅ Supporté" : "❌"} />
              <Row label="GPU" value={model.gpu_supported ? "✅ Supporté" : "❌"} />
              <Row label="Horizon max" value={model.horizon_max.toString()} />
              <Row label="Fréquences" value={model.frequencies} />
            </div>
          </div>
        )}
        {model.installed && (
          <div className="fms-section">
            <span className="fms-section-title">État</span>
            <div className="fms-grid">
              <Row label="Status" value="● Installé" />
              <Row label="Espace utilisé" value={formatSize(model.size_on_disk)} />
            </div>
          </div>
        )}
      </div>
      <div className="fms-footer">
        {!model.is_cloud && !model.installed && (
          <ModelInstallBtn modelId={model.id} onDone={onRefresh} />
        )}
        {model.installed && (
          <button className="fms-uninstall" onClick={() => void handleUninstall()}>
            Désinstaller
          </button>
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
