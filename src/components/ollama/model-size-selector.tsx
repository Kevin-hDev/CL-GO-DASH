import { CaretDown } from "@/components/ui/icons";
import type { OllamaModel } from "@/types/agent";

interface ModelSizeSelectorProps {
  models: OllamaModel[];
  selected: string;
  onSelect: (name: string) => void;
}

export function ModelSizeSelector({ models, selected, onSelect }: ModelSizeSelectorProps) {
  if (models.length <= 1) return null;

  return (
    <div className="relative">
      <select
        value={selected}
        onChange={(e) => onSelect(e.target.value)}
        className="appearance-none bg-[var(--void)] border border-[var(--edge)] rounded px-3 py-1 pr-6 text-xs text-[var(--ink)]"
      >
        {models.map((m) => (
          <option key={m.name} value={m.name}>
            {m.parameter_size} ({(m.size / 1e9).toFixed(1)} Go)
          </option>
        ))}
      </select>
      <CaretDown size={12} className="absolute right-2 top-1/2 -translate-y-1/2 text-[var(--ink-faint)] pointer-events-none" />
    </div>
  );
}
