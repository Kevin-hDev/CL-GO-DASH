import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { RoundToggle } from "@/components/heartbeat/round-toggle";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";
import { SettingsCard } from "./settings-card";
import { SettingsRow } from "./settings-row";
import "./tools-settings.css";

interface AgentSettings {
  permission_mode: string;
  enabled_optional_tools?: string[];
}

interface ToolCatalogEntry {
  id: string;
  locked: boolean;
  defaultEnabled: boolean;
  group: string;
}

export function ToolsSettings() {
  const { t } = useTranslation();
  const [catalog, setCatalog] = useState<ToolCatalogEntry[]>([]);
  const [enabledTools, setEnabledTools] = useState<string[]>([]);

  const load = useCallback(() => {
    void Promise.all([
      invoke<AgentSettings>("get_agent_settings"),
      invoke<ToolCatalogEntry[]>("list_agent_tool_catalog"),
    ])
      .then(([settings, tools]) => {
        setEnabledTools(settings.enabled_optional_tools ?? []);
        setCatalog(tools);
      })
      .catch(() => showToast(i18n.t("errors.operationFailed"), "error"));
  }, []);

  useEffect(() => {
    load();
  }, [load]);

  const enabledSet = useMemo(() => new Set(enabledTools), [enabledTools]);
  const locked = useMemo(() => catalog.filter((tool) => tool.locked), [catalog]);
  const optional = useMemo(() => catalog.filter((tool) => !tool.locked), [catalog]);

  const toggleTool = useCallback((tool: ToolCatalogEntry, enabled: boolean) => {
    const previous = enabledTools;
    const next = enabled
      ? [...new Set([...enabledTools, tool.id])]
      : enabledTools.filter((id) => id !== tool.id);
    setEnabledTools(next);
    invoke<AgentSettings>("set_agent_tool_enabled", { toolId: tool.id, enabled })
      .then((settings) => setEnabledTools(settings.enabled_optional_tools ?? []))
      .catch(() => {
        setEnabledTools(previous);
        showToast(i18n.t("errors.saveFailed"), "error");
      });
  }, [enabledTools]);

  return (
    <div className="ats-page">
      <div className="ats-inner">
        <h2 className="ats-title">{t("settings.tabs.tools")}</h2>

        <h3 className="ats-section-title">{t("settings.tools.lockedTitle")}</h3>
        <SettingsCard>
          {locked.map((tool) => (
            <SettingsRow
              key={tool.id}
              title={tool.id}
              description={t(`settings.tools.descriptions.${tool.id}`)}
              className="ats-tool-row"
            >
              <span className="ats-lock-pill">{t("settings.tools.lockedBadge")}</span>
            </SettingsRow>
          ))}
        </SettingsCard>

        <h3 className="ats-section-title">{t("settings.tools.optionalTitle")}</h3>
        <SettingsCard>
          {optional.map((tool) => {
            const enabled = enabledSet.has(tool.id);
            return (
              <SettingsRow
                key={tool.id}
                title={tool.id}
                description={t(`settings.tools.descriptions.${tool.id}`)}
                className={`ats-tool-row${enabled ? "" : " is-off"}`}
              >
                <RoundToggle
                  checked={enabled}
                  onChange={(checked) => toggleTool(tool, checked)}
                  title={t(`settings.tools.descriptions.${tool.id}`)}
                />
              </SettingsRow>
            );
          })}
        </SettingsCard>
      </div>
    </div>
  );
}
