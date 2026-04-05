import { useTranslation } from "react-i18next";
import { useSettings } from "@/hooks/use-settings";
import type { Theme } from "@/hooks/use-theme";
import { FontSizeSlider } from "./font-size-slider";
import { FontFamilyPicker } from "./font-family-picker";
import { ThemeSelector } from "./theme-selector";
import { LanguagePicker } from "./language-picker";

interface SettingsTabProps {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
}

export function SettingsTab({ theme, onThemeChange }: SettingsTabProps): {
  list: React.ReactNode;
  detail: React.ReactNode;
} {
  const settings = useSettings();
  const { t } = useTranslation();

  const list = (
    <div style={{ padding: 16 }}>
      <span style={{
        fontSize: "var(--text-sm)", fontWeight: 600,
        textTransform: "uppercase", letterSpacing: "0.5px",
        color: "var(--ink-muted)",
      }}>
        {t("settings.title")}
      </span>
    </div>
  );

  const detail = (
    <div style={{ padding: 24, overflowY: "auto", flex: 1 }}>
      <div style={{
        maxWidth: 520, display: "flex", flexDirection: "column", gap: 32,
      }}>
        <ThemeSelector value={theme} onChange={onThemeChange} />
        <FontSizeSlider
          value={settings.fontSize}
          onChange={settings.setFontSize}
          onDecrease={settings.decreaseFont}
          onIncrease={settings.increaseFont}
        />
        <FontFamilyPicker
          value={settings.fontFamilyId}
          onChange={settings.setFontFamily}
        />
        <LanguagePicker />
      </div>
    </div>
  );

  return { list, detail };
}
