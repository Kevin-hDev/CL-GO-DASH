import { idMatch } from "@/lib/utils";
import type { OllamaModel } from "@/types/agent";
import "./ollama.css";

interface ModelfileListProps {
  models: OllamaModel[];
  selectedModel: string | null;
  onSelect: (name: string) => void;
}

export function ModelfileList({ models, selectedModel, onSelect }: ModelfileListProps) {
  return (
    <div style={{ flex: 1, overflowY: "auto", padding: "var(--space-sm)", paddingBottom: 20 }}>
      {models.map((m: OllamaModel) => (
        <div
          key={m.name}
          className={`ollama-model-item ${selectedModel != null && idMatch(selectedModel, m.name) ? "active" : ""}`}
          onClick={() => onSelect(m.name)}
        >
          {m.name}
        </div>
      ))}
    </div>
  );
}
