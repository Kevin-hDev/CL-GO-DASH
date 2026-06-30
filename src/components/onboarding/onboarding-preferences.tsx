import { useTranslation } from "react-i18next";
import type { ReactNode } from "react";
import type { ThemeChoice } from "@/hooks/use-theme";
import { Gear, Moon, Sun, CaretRight } from "@/components/ui/icons";
import { SettingsSelect } from "@/components/settings/settings-select";
import { LANGUAGE_OPTIONS } from "@/components/settings/general-settings-options";

interface OnboardingPreferencesProps {
  themeChoice: ThemeChoice;
  onThemeChange: (theme: ThemeChoice) => void;
  onNext: () => void;
}

const THEME_OPTIONS: { id: ThemeChoice; labelKey: string; icon: ReactNode }[] = [
  { id: "light", labelKey: "settings.light", icon: <Sun size="var(--icon-lg)" /> },
  { id: "dark", labelKey: "settings.dark", icon: <Moon size="var(--icon-lg)" /> },
  { id: "system", labelKey: "settings.system", icon: <Gear size="var(--icon-lg)" /> },
];

export function OnboardingPreferences({
  themeChoice,
  onThemeChange,
  onNext,
}: OnboardingPreferencesProps) {
  const { t, i18n } = useTranslation();

  const changeLang = (lang: string) => {
    void i18n.changeLanguage(lang);
    localStorage.setItem("clgo-language", lang);
  };

  return (
    <div className="ob-page">
      <div className="ob-copy">
        <h1 className="ob-title">{t("onboarding.preferences.title")}</h1>
        <p className="ob-description">{t("onboarding.preferences.description")}</p>
      </div>

      <div className="ob-settings-panel">
        <div className="ob-field">
          <span className="ob-field-label">{t("settings.general.themeTitle")}</span>
          <div className="ob-theme-grid">
            {THEME_OPTIONS.map((option) => (
              <button
                key={option.id}
                type="button"
                className={`ob-theme-choice ${themeChoice === option.id ? "is-active" : ""}`}
                onClick={() => onThemeChange(option.id)}
              >
                {option.icon}
                <span>{t(option.labelKey)}</span>
              </button>
            ))}
          </div>
        </div>

        <div className="ob-field">
          <span className="ob-field-label">{t("settings.general.languageTitle")}</span>
          <SettingsSelect
            options={LANGUAGE_OPTIONS}
            value={i18n.language}
            onChange={changeLang}
          />
        </div>
      </div>

      <button type="button" className="ob-primary-btn" onClick={onNext}>
        {t("onboarding.common.continue")}
        <CaretRight size="var(--icon-md)" weight="bold" />
      </button>
    </div>
  );
}
