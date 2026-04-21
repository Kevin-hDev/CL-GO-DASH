import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { useApiKeys } from "@/hooks/use-api-keys";
import type { ProviderSpec } from "@/types/api";
import { ApiKeysSidebar } from "./api-keys-sidebar";
import { ApiKeysDetails } from "./api-keys-details";
import { ApiKeysConfigDialog } from "./api-keys-config-dialog";
import { ConnectorsModal } from "./connectors-modal";
import { EmptyState } from "@/components/ui/empty-state";
import "./api-keys.css";
import "./api-keys-main.css";
import "./api-keys-detail.css";
import "./api-keys-dialog.css";
import "./connectors-modal.css";
import "./connector-card.css";

type DialogState =
  | { kind: "none" }
  | { kind: "connectors" }
  | {
      kind: "config";
      provider: ProviderSpec;
      alreadyConfigured: boolean;
      returnTo: "connectors" | "none";
    };

export function ApiKeysTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const { catalog, configuredIds, configured, setKey, deleteKey, testKey } =
    useApiKeys();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [dialog, setDialog] = useState<DialogState>({ kind: "none" });

  useEffect(() => {
    if (selectedId === null && configured.length > 0) {
      setSelectedId(configured[0].id);
    }
  }, [selectedId, configured]);

  const selected = selectedId
    ? configured.find((p) => p.id === selectedId) ?? null
    : null;

  const list = (
    <ApiKeysSidebar
      configured={configured}
      selectedId={selectedId}
      onSelect={setSelectedId}
    />
  );

  const handleDelete = async () => {
    if (!selected) return;
    const id = selected.id;
    setSelectedId(null);
    await deleteKey(id);
  };

  const handleConfigClose = () => {
    if (dialog.kind === "config" && dialog.returnTo === "connectors") {
      setDialog({ kind: "connectors" });
    } else {
      setDialog({ kind: "none" });
    }
  };

  const detail = (
    <>
      {selected ? (
        <ApiKeysDetails
          provider={selected}
          onEdit={() =>
            setDialog({
              kind: "config",
              provider: selected,
              alreadyConfigured: true,
              returnTo: "none",
            })
          }
          onDelete={handleDelete}
          onAddConnector={() => setDialog({ kind: "connectors" })}
        />
      ) : (
        <div style={{ padding: 24, flex: 1, display: "flex", flexDirection: "column" }}>
          <div style={{ maxWidth: 600, width: "100%", margin: "0 auto" }}>
            <div style={{
              display: "flex", alignItems: "center",
              justifyContent: "space-between", marginBottom: 28,
            }}>
              <h2 style={{ fontSize: "var(--text-xl)", fontWeight: 700, color: "var(--ink)", margin: 0 }}>
                {t("apiKeys.main.title")}
              </h2>
              <button type="button" className="ak-connectors-btn" onClick={() => setDialog({ kind: "connectors" })}>
                <Plus size={14} weight="bold" />
                {t("apiKeys.main.connectorsBtn")}
              </button>
            </div>
          </div>
          <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center" }}>
            <EmptyState message={t("apiKeys.empty.title")} />
          </div>
        </div>
      )}

      {dialog.kind === "connectors" && (
        <ConnectorsModal
          catalog={catalog}
          configuredIds={configuredIds}
          onPick={(p) =>
            setDialog({
              kind: "config",
              provider: p,
              alreadyConfigured: false,
              returnTo: "connectors",
            })
          }
          onClose={() => setDialog({ kind: "none" })}
        />
      )}

      {dialog.kind === "config" && (
        <ApiKeysConfigDialog
          provider={dialog.provider}
          alreadyConfigured={dialog.alreadyConfigured}
          onClose={handleConfigClose}
          onSave={async (key) => {
            await setKey(dialog.provider.id, key);
          }}
          onTest={async () => {
            await testKey(dialog.provider.id);
          }}
          onClearKey={
            dialog.alreadyConfigured
              ? async () => {
                  await deleteKey(dialog.provider.id);
                  setSelectedId(null);
                }
              : undefined
          }
        />
      )}
    </>
  );

  return { list, detail };
}
