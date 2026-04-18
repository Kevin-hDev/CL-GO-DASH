import { CaretLeft, CaretRight } from "@phosphor-icons/react";
import { FONT_SIZES, type FontSize } from "@/hooks/use-settings";

interface FontSizeSliderProps {
  value: FontSize;
  onChange: (size: FontSize) => void;
  onDecrease: () => void;
  onIncrease: () => void;
}

export function FontSizeSlider({ value, onChange, onDecrease, onIncrease }: FontSizeSliderProps) {
  return (
    <div>
      <label style={{
        display: "block", fontSize: "var(--text-xs)", color: "var(--ink-muted)",
        textTransform: "uppercase", letterSpacing: "0.5px", marginBottom: 12,
      }}>
        Font size — {value}%
      </label>
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        <button onClick={onDecrease} style={arrowStyle}>
          <CaretLeft size={16} weight="bold" />
        </button>
        <div style={{ flex: 1, display: "flex", gap: 4 }}>
          {FONT_SIZES.map((size) => (
            <button
              key={size}
              onClick={() => onChange(size)}
              style={{
                flex: 1, padding: "10px 0",
                fontSize: "var(--text-xs)", fontFamily: "var(--font-mono)",
                borderRadius: "var(--radius-sm)", cursor: "pointer",
                border: size === value ? "1px solid var(--pulse)" : "1px solid var(--edge)",
                color: size === value ? "var(--select-text)" : "var(--ink-muted)",
                background: size === value ? "var(--pulse-muted)" : "transparent",
                transition: "all 200ms ease-out",
              }}
            >
              {size}%
            </button>
          ))}
        </div>
        <button onClick={onIncrease} style={arrowStyle}>
          <CaretRight size={16} weight="bold" />
        </button>
      </div>
    </div>
  );
}

const arrowStyle: React.CSSProperties = {
  display: "flex", alignItems: "center", justifyContent: "center",
  width: 32, height: 32, borderRadius: "var(--radius-sm)",
  border: "1px solid var(--edge)", background: "transparent",
  color: "var(--ink-muted)", cursor: "pointer",
};
