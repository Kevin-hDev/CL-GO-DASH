import type { Theme } from "@/hooks/use-theme";
import type { useSettings } from "@/hooks/use-settings";
import { FontSizeSlider } from "./font-size-slider";
import { FontFamilyPicker } from "./font-family-picker";
import { ThemeSelector } from "./theme-selector";
import { LanguagePicker } from "./language-picker";

interface GeneralSettingsProps {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
  settings: ReturnType<typeof useSettings>;
}

export function GeneralSettings({ theme, onThemeChange, settings }: GeneralSettingsProps) {
  return (
    <div style={{ padding: 24, overflowY: "auto", flex: 1, display: "flex", justifyContent: "center" }}>
      <div style={{
        maxWidth: 520, width: "100%", display: "flex", flexDirection: "column", gap: 32,
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
}
