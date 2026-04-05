import { Check } from "@/components/ui/icons";
import type { Theme } from "@/hooks/use-theme";

interface ThemeSelectorProps {
  value: Theme;
  onChange: (theme: Theme) => void;
}

const THEMES: { id: Theme; label: string; bg: string; accent: string; text: string }[] = [
  { id: "light", label: "Light", bg: "#f5f3f0", accent: "#ea6c10", text: "#1a1a1a" },
  { id: "dark", label: "Dark", bg: "#1c1c22", accent: "#f97316", text: "#e8e6e3" },
  { id: "orange", label: "Orange", bg: "#2b1508", accent: "#f97316", text: "#f8f0e4" },
];

export function ThemeSelector({ value, onChange }: ThemeSelectorProps) {
  return (
    <div>
      <label style={{
        display: "block", fontSize: "var(--text-xs)", color: "var(--ink-muted)",
        textTransform: "uppercase", letterSpacing: "0.5px", marginBottom: 12,
      }}>
        Theme
      </label>
      <div style={{ display: "flex", gap: 12 }}>
        {THEMES.map((t) => {
          const active = t.id === value;
          return (
            <div
              key={t.id}
              onClick={() => onChange(t.id)}
              style={{
                flex: 1, padding: 16, borderRadius: "var(--radius-md)",
                cursor: "pointer", position: "relative",
                border: active ? "2px solid var(--pulse)" : "1px solid var(--edge)",
                transition: "all 200ms ease-out",
              }}
            >
              {/* Preview swatch */}
              <div style={{
                height: 48, borderRadius: "var(--radius-sm)", marginBottom: 10,
                background: t.bg, border: "1px solid rgba(128,128,128,0.15)",
                display: "flex", alignItems: "flex-end", padding: 6, gap: 4,
              }}>
                <div style={{ width: 20, height: 4, borderRadius: 2, background: t.accent }} />
                <div style={{ width: 12, height: 4, borderRadius: 2, background: t.text, opacity: 0.3 }} />
              </div>
              <div style={{
                fontSize: "var(--text-sm)", fontWeight: active ? 600 : 400,
                color: active ? "var(--pulse)" : "var(--ink)",
                textAlign: "center",
              }}>
                {t.label}
              </div>
              {active && (
                <div style={{
                  position: "absolute", top: 8, right: 8,
                  color: "var(--pulse)",
                }}>
                  <Check size={16} weight="bold" />
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}
