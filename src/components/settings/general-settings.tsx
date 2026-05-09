import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import type { ThemeChoice } from "@/hooks/use-theme";
import type { useSettings } from "@/hooks/use-settings";
import { FONT_SIZES, FONT_FAMILIES } from "@/hooks/use-settings";
import { RoundToggle } from "@/components/heartbeat/round-toggle";
import { showToast } from "@/lib/toast-emitter";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import { SettingsSelect, type SelectOption } from "./settings-select";
import { ThemeSelector } from "./theme-selector";

interface GeneralSettingsProps {
  themeChoice: ThemeChoice;
  onThemeChange: (theme: ThemeChoice) => void;
  settings: ReturnType<typeof useSettings>;
}

interface StartupState {
  autostart: boolean;
  start_hidden: boolean;
  response_language: string;
  link_preview_enabled: boolean;
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
  { value: "fr", label: "Français" },
  { value: "de", label: "Deutsch" },
  { value: "es", label: "Español" },
  { value: "it", label: "Italiano" },
  { value: "zh", label: "中文" },
  { value: "ja", label: "日本語" },
];

const RESPONSE_LANGUAGE_OPTIONS: SelectOption[] = [
  { value: "", label: "—" },
  { value: "English", label: "English" },
  { value: "French", label: "Français" },
  { value: "German", label: "Deutsch" },
  { value: "Spanish", label: "Español" },
  { value: "Italian", label: "Italiano" },
  { value: "Chinese", label: "中文" },
  { value: "Japanese", label: "日本語" },
];

export function GeneralSettings({ themeChoice, onThemeChange, settings }: GeneralSettingsProps) {
  const { t, i18n } = useTranslation();
  const [startup, setStartup] = useState<StartupState>({
    autostart: false,
    start_hidden: false,
    response_language: "",
    link_preview_enabled: true,
  });

  useEffect(() => {
    invoke<Record<string, unknown>>("get_advanced_settings")
      .then((s) => {
        const linkPreview = (s.link_preview_enabled as boolean) ?? true;
        localStorage.setItem("clgo-link-preview", String(linkPreview));
        setStartup({
          autostart: s.autostart as boolean ?? false,
          start_hidden: s.start_hidden as boolean ?? false,
          response_language: s.response_language as string ?? "",
          link_preview_enabled: linkPreview,
        });
      })
      .catch(() => console.warn("Failed to load advanced settings"));
  }, []);

  const saveAdvanced = useCallback((patch: Partial<StartupState>) => {
    setStartup((prev) => {
      const next = { ...prev, ...patch };
      invoke("patch_advanced_settings", { patch }).catch(() => showToast(t("errors.saveFailed"), "error"));
      if ("link_preview_enabled" in patch) {
        localStorage.setItem("clgo-link-preview", String(patch.link_preview_enabled));
      }
      return next;
    });
  }, []);

  const changeLang = (lang: string) => {
    void i18n.changeLanguage(lang);
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
            <ThemeSelector value={themeChoice} onChange={onThemeChange} />
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

          <SettingsRow
            title={t("settings.general.responseLangTitle")}
            description={t("settings.general.responseLangDesc")}
          >
            <SettingsSelect
              options={RESPONSE_LANGUAGE_OPTIONS}
              value={startup.response_language}
              onChange={(v) => saveAdvanced({ response_language: v })}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.general.sidebarExpandTitle")}
            description={t("settings.general.sidebarExpandDesc")}
          >
            <RoundToggle
              checked={settings.sidebarExpand}
              onChange={settings.setSidebarExpand}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.general.linkPreviewTitle")}
            description={t("settings.general.linkPreviewDesc")}
          >
            <RoundToggle
              checked={startup.link_preview_enabled}
              onChange={(v) => saveAdvanced({ link_preview_enabled: v })}
            />
          </SettingsRow>
        </SettingsCard>

        <SettingsCard>
          <SettingsRow
            title={t("settings.advanced.autostartTitle")}
            description={t("settings.advanced.autostartDesc")}
          >
            <RoundToggle
              checked={startup.autostart}
              onChange={(v) => saveAdvanced({ autostart: v })}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.advanced.startHiddenTitle")}
            description={t("settings.advanced.startHiddenDesc")}
          >
            <RoundToggle
              checked={startup.start_hidden}
              onChange={(v) => saveAdvanced({ start_hidden: v })}
            />
          </SettingsRow>
        </SettingsCard>
      </div>
    </div>
  );
}
