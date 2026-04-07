import { useOllamaModels } from "@/hooks/use-ollama-models";
import { idMatch } from "@/lib/utils";
import type { OllamaModel } from "@/types/agent";
import "./ollama.css";

interface ModelfileListProps {
  selectedModel: string | null;
  onSelect: (name: string) => void;
}

export function ModelfileList({ selectedModel, onSelect }: ModelfileListProps) {
  const { models } = useOllamaModels();

  return (
    <div style={{ flex: 1, overflowY: "auto", padding: "var(--space-sm)" }}>
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
