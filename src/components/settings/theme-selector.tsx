import { useTranslation } from "react-i18next";
import { Check } from "@/components/ui/icons";
import type { Theme } from "@/hooks/use-theme";

interface ThemeSelectorProps {
  value: Theme;
  onChange: (theme: Theme) => void;
}

const THEMES: { id: Theme; i18n: string; bg: string; accent: string; text: string }[] = [
  { id: "light", i18n: "settings.light", bg: "#f5f3f0", accent: "#ea6c10", text: "#1a1a1a" },
  { id: "dark", i18n: "settings.dark", bg: "#1c1c22", accent: "#f97316", text: "#e8e6e3" },
];

export function ThemeSelector({ value, onChange }: ThemeSelectorProps) {
  const { t } = useTranslation();

  return (
    <div style={{ display: "flex", gap: 10 }}>
      {THEMES.map((theme) => {
        const active = theme.id === value;
        return (
          <div
            key={theme.id}
            onClick={() => onChange(theme.id)}
            style={{
              width: 100,
              padding: 10,
              borderRadius: "var(--radius-sm)",
              cursor: "pointer",
              position: "relative",
              border: `1px solid ${active ? "var(--ink-faint)" : "var(--edge)"}`,
              background: active ? "var(--surface)" : "transparent",
              transition: "all 200ms ease-out",
            }}
          >
            <div style={{
              height: 36,
              borderRadius: 4,
              marginBottom: 6,
              background: theme.bg,
              border: "1px solid rgba(128,128,128,0.15)",
              display: "flex",
              alignItems: "flex-end",
              padding: 4,
              gap: 3,
            }}>
              <div style={{ width: 14, height: 3, borderRadius: 2, background: theme.accent }} />
              <div style={{ width: 8, height: 3, borderRadius: 2, background: theme.text, opacity: 0.3 }} />
            </div>
            <div style={{
              fontSize: "var(--text-xs)",
              fontWeight: active ? 600 : 400,
              color: active ? "var(--ink)" : "var(--ink-muted)",
              textAlign: "center",
            }}>
              {t(theme.i18n)}
            </div>
            {active && (
              <div style={{
                position: "absolute",
                top: 6,
                right: 6,
                color: "var(--ink)",
              }}>
                <Check size={12} weight="bold" />
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
