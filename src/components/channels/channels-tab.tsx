import { useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { useChannels } from "@/hooks/use-channels";
import type { ChannelType } from "@/types/channels";
import { ChannelsSidebar } from "./channels-sidebar";
import { ChannelsDetail } from "./channels-detail";
import { ChannelsBrowseModal } from "./channels-browse-modal";
import { ChannelsConfigDialog } from "./channels-config-dialog";
import { EmptyState } from "@/components/ui/empty-state";
import "./channels.css";

type DialogState =
  | { kind: "none" }
  | { kind: "browse" }
  | { kind: "config"; channelId: ChannelType; returnTo: "browse" | "none" };

export function ChannelsTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const { health, config, saveConfig, refreshHealth } = useChannels();
  const [selectedKey, setSelectedKey] = useState<string | null>(null);
  const [dialog, setDialog] = useState<DialogState>({ kind: "none" });

  const configuredAccounts = useMemo(() => {
    if (!config) return [];
    return (["telegram", "slack", "discord"] as ChannelType[]).flatMap((ch) =>
      (config.channels[ch] ?? []).map((acc) => ({ channelId: ch, accountId: acc.account_id, config: acc })),
    );
  }, [config]);

  useEffect(() => {
    if (selectedKey === null && configuredAccounts.length > 0) {
      setSelectedKey(`${configuredAccounts[0].channelId}:${configuredAccounts[0].accountId}`);
    }
  }, [selectedKey, configuredAccounts]);

  const selected = selectedKey ? configuredAccounts.find((a) => `${a.channelId}:${a.accountId}` === selectedKey) ?? null : null;

  const handlePick = (channelId: ChannelType) => {
    setDialog({ kind: "config", channelId, returnTo: "browse" });
  };

  const handleConfigSaved = async (channelId: ChannelType, accountId: string) => {
    if (!config) return;
    const list = [...(config.channels[channelId] ?? [])];
    if (!list.some((a) => a.account_id === accountId)) {
      const hasDefaultModel = Boolean(config.default_provider && config.default_model);
      list.push({
        account_id: accountId,
        enabled: hasDefaultModel,
        allowlist: [],
        require_mention: true,
        provider: config.default_provider,
        model: config.default_model,
      });
      await saveConfig({ ...config, channels: { ...config.channels, [channelId]: list } });
    }
    setSelectedKey(`${channelId}:${accountId}`);
    setDialog({ kind: "none" });
    await refreshHealth();
  };

  const list = (
    <ChannelsSidebar
      accounts={configuredAccounts}
      healthEntries={health.channels}
      selectedKey={selectedKey}
      onSelect={setSelectedKey}
    />
  );

  const browseHeader = (
    <div className="ct-browse-header">
      <p className="ct-subtitle">{t("channels.main.subtitle")}</p>
      <button type="button" className="ak-connectors-btn" onClick={() => setDialog({ kind: "browse" })}>
        <Plus size={14} weight="bold" />
        {t("channels.main.browseBtn")}
      </button>
    </div>
  );

  const detail = (
    <>
      {selected && config ? (
        <div className="ct-detail-wrapper">
          {browseHeader}
          <ChannelsDetail
            channelId={selected.channelId}
            account={selected.config}
            status={health.channels.find((c) => c.channel_id === selected.channelId && c.account_id === selected.accountId)}
            config={config}
            onSaveConfig={saveConfig}
            onDelete={() => {
              setSelectedKey(null);
              void refreshHealth();
            }}
          />
        </div>
      ) : (
        <div className="ct-empty-wrapper">
          {browseHeader}
          <div className="ct-empty-center">
            <EmptyState message={t("channels.sidebar.empty")} />
          </div>
        </div>
      )}

      {dialog.kind === "browse" && (
        <ChannelsBrowseModal
          onPick={handlePick}
          onClose={() => setDialog({ kind: "none" })}
        />
      )}
      {dialog.kind === "config" && (
        <ChannelsConfigDialog
          channelId={dialog.channelId}
          onClose={() => setDialog(dialog.returnTo === "browse" ? { kind: "browse" } : { kind: "none" })}
          onSaved={(accountId: string) => void handleConfigSaved(dialog.channelId, accountId)}
        />
      )}
    </>
  );

  return { list, detail };
}
