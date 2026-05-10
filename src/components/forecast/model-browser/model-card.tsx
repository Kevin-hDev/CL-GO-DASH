import { ModelInstallBtn } from "./model-install-btn";

interface ForecastModel {
  id: string;
  display_name: string;
  params: string;
  size_mb: number;
  ram_mb: number;
  cpu_supported: boolean;
  gpu_supported: boolean;
  is_cloud: boolean;
  installed: boolean;
}

interface ModelCardProps {
  model: ForecastModel;
  onSelect: () => void;
  onRefresh: () => void;
}

export function ModelCard({ model, onSelect, onRefresh }: ModelCardProps) {
  return (
    <div className="fmc-card">
      <button className="fmc-info" onClick={onSelect}>
        <span className="fmc-name">{model.display_name}</span>
        <span className="fmc-meta">
          {model.is_cloud ? (
            <span className="fmc-cloud">☁ Cloud</span>
          ) : (
            <>
              {model.params} · {model.size_mb} MB
              {model.cpu_supported && " · CPU ✓"}
              {model.gpu_supported && " · GPU ✓"}
            </>
          )}
        </span>
        {!model.is_cloud && (
          <span className="fmc-ram">RAM: ~{model.ram_mb} MB</span>
        )}
      </button>
      <div className="fmc-actions">
        {model.is_cloud ? (
          <span className="fmc-cloud-badge">☁</span>
        ) : model.installed ? (
          <span className="fmc-installed">Installé</span>
        ) : (
          <ModelInstallBtn modelId={model.id} onDone={onRefresh} />
        )}
      </div>
    </div>
  );
}
