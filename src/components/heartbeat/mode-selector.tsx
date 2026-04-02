import "./mode-selector.css";

const MODES = ["auto", "explorer", "free", "evolve"] as const;

interface ModeSelectorProps {
  value: string;
  onChange: (mode: string) => void;
}

export function ModeSelector({ value, onChange }: ModeSelectorProps) {
  return (
    <div className="mode-selector">
      {MODES.map((mode) => (
        <button
          key={mode}
          className={`mode-option ${value.includes(mode) ? "active" : ""}`}
          onClick={() => onChange(mode)}
          type="button"
        >
          --{mode}
        </button>
      ))}
    </div>
  );
}
