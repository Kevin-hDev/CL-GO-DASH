import { useState, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useAvailableModels } from "@/hooks/use-available-models";
import { RoundToggle } from "@/components/heartbeat/round-toggle";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import { SettingsSelect, type SelectGroup } from "./settings-select";
import { PathListEditor } from "./path-list-editor";
import { OllamaSettingsSection } from "./ollama-settings-section";

interface AdvancedState {
  autostart: boolean;
  start_hidden: boolean;
  show_tray: boolean;
  default_model: string;
  keep_alive: string;
  allowed_paths: string[];
  hardware_accel: string;
  multi_model: boolean;
  show_gpu_status: boolean;
}

const DEFAULTS: AdvancedState = {
  autostart: false,
  start_hidden: false,
  show_tray: true,
  default_model: "",
  keep_alive: "5m",
  allowed_paths: ["/"],
  hardware_accel: "gpu",
  multi_model: false,
  show_gpu_status: false,
};

export function AdvancedSettings() {
  const { t } = useTranslation();
  const { groups } = useAvailableModels();
  const [state, setState] = useState<AdvancedState>(DEFAULTS);

  useEffect(() => {
    invoke<AdvancedState>("get_advanced_settings")
      .then(setState)
      .catch(() => {});
  }, []);

  const save = useCallback((patch: Partial<AdvancedState>) => {
    setState((prev) => {
      const next = { ...prev, ...patch };
      invoke("set_advanced_settings", { settings: next }).catch(() => {});
      return next;
    });
  }, []);

  const modelGroups = useMemo((): SelectGroup[] => {
    const result: SelectGroup[] = [];
    for (const [, models] of groups) {
      if (models.length === 0) continue;
      result.push({
        label: models[0].provider_name,
        options: models.map((m) => ({
          value: `${m.provider_id}:${m.id}`,
          label: m.id,
          dimmed: !m.is_free && !m.is_local,
        })),
      });
    }
    return result;
  }, [groups]);

  const titleStyle = { fontSize: "var(--text-xl)", fontWeight: 700, color: "var(--ink)", marginBottom: 28 } as const;
  const subStyle = { fontSize: "var(--text-base)", fontWeight: 600, color: "var(--ink)", marginTop: 28, marginBottom: 12 } as const;

  return (
    <div style={{ padding: 24, overflowY: "auto", flex: 1 }}>
      <div style={{ maxWidth: 600, width: "100%", margin: "0 auto" }}>
        <h2 style={titleStyle}>{t("settings.tabs.advanced")}</h2>

        <SettingsCard>
          <SettingsRow
            title={t("settings.advanced.autostartTitle")}
            description={t("settings.advanced.autostartDesc")}
          >
            <RoundToggle
              checked={state.autostart}
              onChange={(v) => save({ autostart: v })}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.advanced.startHiddenTitle")}
            description={t("settings.advanced.startHiddenDesc")}
          >
            <RoundToggle
              checked={state.start_hidden}
              onChange={(v) => save({ start_hidden: v })}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.advanced.trayTitle")}
            description={t("settings.advanced.trayDesc")}
          >
            <RoundToggle
              checked={state.show_tray}
              onChange={(v) => save({ show_tray: v })}
            />
          </SettingsRow>

          <SettingsRow
            title={t("settings.advanced.defaultModelTitle")}
            description={t("settings.advanced.defaultModelDesc")}
          >
            <SettingsSelect
              groups={modelGroups}
              value={state.default_model}
              onChange={(v) => save({ default_model: v })}
              searchable
              searchPlaceholder={t("settings.advanced.searchModel")}
            />
          </SettingsRow>

        </SettingsCard>

        <h3 style={subStyle}>{t("settings.advanced.ollamaTitle")}</h3>

        <OllamaSettingsSection
          keepAlive={state.keep_alive}
          hardwareAccel={state.hardware_accel}
          multiModel={state.multi_model}
          showGpuStatus={state.show_gpu_status}
          onSave={save}
        />

        <h3 style={subStyle}>{t("settings.advanced.fileAccessTitle")}</h3>

        <SettingsCard>
          <div style={{ padding: "14px 20px" }}>
            <div style={{ fontSize: "var(--text-xs)", color: "var(--ink-muted)", marginBottom: 12 }}>
              {t("settings.advanced.fileAccessDesc")}
            </div>
            <PathListEditor
              paths={state.allowed_paths}
              onChange={(paths) => save({ allowed_paths: paths })}
            />
          </div>
        </SettingsCard>
      </div>
    </div>
  );
}