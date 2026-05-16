import { idMatch } from "@/lib/utils";
import type { ForecastModelEntry } from "../forecast-model-meta";
import "@/components/ollama/ollama.css";

interface ForecastConfigListProps {
  models: ForecastModelEntry[];
  selectedModelId: string | null;
  onSelect: (modelId: string) => void;
}

export function ForecastConfigList({
  models,
  selectedModelId,
  onSelect,
}: ForecastConfigListProps) {
  return (
    <div className="fcl-root">
      {models.map((model) => (
        <div
          key={model.id}
          className={`ollama-model-item ${selectedModelId != null && idMatch(selectedModelId, model.id) ? "active" : ""}`}
          role="button"
          tabIndex={0}
          onClick={() => onSelect(model.id)}
          onKeyDown={(event) => {
            if (event.key === "Enter" || event.key === " ") onSelect(model.id);
          }}
        >
          {model.display_name}
        </div>
      ))}
    </div>
  );
}
