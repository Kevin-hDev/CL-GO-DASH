const MODES = ["auto", "explorer", "free", "evolve"] as const;

interface ModeSelectorProps {
  value: string;
  onChange: (mode: string) => void;
}

export function ModeSelector({ value, onChange }: ModeSelectorProps) {
  return (
    <div style={{ display: "flex", gap: 6 }}>
      {MODES.map((mode) => {
        const active = value.includes(mode);
        return (
          <button
            key={mode}
            type="button"
            onClick={() => onChange(mode)}
            style={{
              padding: "8px 14px",
              fontSize: "var(--text-sm)",
              fontFamily: "var(--font-mono)",
              borderRadius: "var(--radius-sm)",
              cursor: "pointer",
              border: `1px solid ${active ? "var(--pulse)" : "var(--edge)"}`,
              color: active ? "var(--pulse)" : "var(--ink-muted)",
              background: active ? "var(--pulse-muted)" : "transparent",
              transition: "all 200ms ease-out",
            }}
          >
            --{mode}
          </button>
        );
      })}
    </div>
  );
}
