import { useTranslation } from "react-i18next";
import { CaretDown } from "@/components/ui/icons";
import type { OllamaModel } from "@/types/agent";
import "./model-size-selector.css";

interface ModelSizeSelectorProps {
  models: OllamaModel[];
  selected: string;
  onSelect: (name: string) => void;
}

export function ModelSizeSelector({ models, selected, onSelect }: ModelSizeSelectorProps) {
  const { t } = useTranslation();
  if (models.length <= 1) return null;

  return (
    <div className="relative">
      <select
        value={selected}
        onChange={(e) => onSelect(e.target.value)}
        className="mss-select"
      >
        {models.map((m) => (
          <option key={m.name} value={m.name}>
            {m.parameter_size} ({(m.size / 1e9).toFixed(1)} {t("units.gb")})
          </option>
        ))}
      </select>
      <CaretDown size="var(--icon-xs)" className="absolute right-2 top-1/2 -translate-y-1/2 mss-caret pointer-events-none" />
    </div>
  );
}
