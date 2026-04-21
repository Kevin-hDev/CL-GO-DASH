import { useTranslation } from "react-i18next";
import type { Theme } from "@/hooks/use-theme";
import type { useSettings } from "@/hooks/use-settings";
import { FONT_SIZES, FONT_FAMILIES } from "@/hooks/use-settings";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import { SettingsSelect, type SelectOption } from "./settings-select";
import { ThemeSelector } from "./theme-selector";

interface GeneralSettingsProps {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
  settings: ReturnType<typeof useSettings>;
}

const FONT_SIZE_OPTIONS: SelectOption[] = FONT_SIZES.map((s) => ({
  value: String(s),
  label: `${s}%`,
}));

const FONT_FAMILY_OPTIONS: SelectOption[] = FONT_FAMILIES.map((f) => ({
  value: f.id,
  label: f.label,
}));

const LANGUAGE_OPTIONS: SelectOption[] = [
  { value: "en", label: "English" },
  { value: "fr", label: "Francais" },
];

export function GeneralSettings({ theme, onThemeChange, settings }: GeneralSettingsProps) {
  const { t, i18n } = useTranslation();

  const changeLang = (lang: string) => {
    i18n.changeLanguage(lang);
    localStorage.setItem("clgo-language", lang);
  };

  return (
    <div style={{ padding: 24, overflowY: "auto", flex: 1 }}>
      <div style={{ maxWidth: 600, width: "100%", margin: "0 auto" }}>
        <h2 style={{
          fontSize: "var(--text-xl)",
          fontWeight: 700,
          color: "var(--ink)",
          marginBottom: 28,
        }}>
          {t("settings.tabs.general")}
        </h2>

        <SettingsCard>
          <SettingsRow
            title={t("settings.general.themeTitle")}
            description={t("settings.general.themeDesc")}
          >
            <ThemeSelector value={theme} onChange={onThemeChange} />
          </SettingsRow>

          <SettingsRow
            title={t("settings.general.fontSizeTitle")}
            description={t("settings.general.fontSizeDesc")}
          >
            <SettingsSelect
              options={FONT_SIZE_OPTIONS}
              value={String(settings.fontSize)}
              onChange={(v) => settings.setFontSize(Number(v) as typeof settings.fontSize)}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.general.fontFamilyTitle")}
            description={t("settings.general.fontFamilyDesc")}
          >
            <SettingsSelect
              options={FONT_FAMILY_OPTIONS}
              value={settings.fontFamilyId}
              onChange={(v) => settings.setFontFamily(v as typeof settings.fontFamilyId)}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.general.languageTitle")}
            description={t("settings.general.languageDesc")}
          >
            <SettingsSelect
              options={LANGUAGE_OPTIONS}
              value={i18n.language}
              onChange={changeLang}
            />
          </SettingsRow>
        </SettingsCard>
      </div>
    </div>
  );
}
