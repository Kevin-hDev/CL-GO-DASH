import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { Plus } from "@/components/ui/icons";
import { useConnectors } from "@/hooks/use-connectors";
import type { McpConnectorSpec } from "@/types/mcp";
import { ConnectorsSidebar } from "./connectors-sidebar";
import { ConnectorsDetail } from "./connectors-detail";
import { McpBrowseModal } from "./mcp-browse-modal";
import { McpConfigDialog } from "./mcp-config-dialog";
import { McpOauthDialog } from "./mcp-oauth-dialog";
import { EmptyState } from "@/components/ui/empty-state";
import "./connectors-tab.css";

type DialogState =
  | { kind: "none" }
  | { kind: "browse" }
  | { kind: "config"; connector: McpConnectorSpec; returnTo: "browse" | "none" }
  | { kind: "oauth-pending"; connector: McpConnectorSpec; returnTo: "browse" | "none" }
  | { kind: "confirm-add"; connector: McpConnectorSpec; returnTo: "browse" | "none" }
  | { kind: "confirm-disconnect"; connectorId: string }
  | { kind: "confirm-delete"; connectorId: string };

export function ConnectorsTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const { catalog, configured, configuredIds, addConnector, removeConnector, toggleStatus } =
    useConnectors();
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [dialog, setDialog] = useState<DialogState>({ kind: "none" });

  useEffect(() => {
    if (selectedId === null && configured.length > 0) setSelectedId(configured[0].id);
  }, [selectedId, configured]);

  const selected = selectedId ? configured.find((c) => c.id === selectedId) ?? null : null;

  const handlePick = (spec: McpConnectorSpec) => {
    if (spec.auth_type === "none") {
      setDialog({ kind: "confirm-add", connector: spec, returnTo: "browse" });
    } else if (spec.auth_type === "oauth") {
      setDialog({ kind: "oauth-pending", connector: spec, returnTo: "browse" });
    } else {
      setDialog({ kind: "config", connector: spec, returnTo: "browse" });
    }
  };

  const handleDelete = async () => {
    if (dialog.kind !== "confirm-delete") return;
    setSelectedId(null);
    await removeConnector(dialog.connectorId);
    setDialog({ kind: "none" });
  };

  const handleDisconnect = async () => {
    if (dialog.kind !== "confirm-disconnect") return;
    await toggleStatus(dialog.connectorId);
    setDialog({ kind: "none" });
  };

  const list = <ConnectorsSidebar configured={configured} selectedId={selectedId} onSelect={setSelectedId} />;

  const browseHeader = (
    <div className="ct-browse-header">
      <p className="ct-subtitle">{t("connectors.main.subtitle")}</p>
      <button type="button" className="ak-connectors-btn" onClick={() => setDialog({ kind: "browse" })}>
        <Plus size={14} weight="bold" />
        {t("connectors.main.browseBtn")}
      </button>
    </div>
  );

  const detail = (
    <>
      {selected ? (
        <div className="ct-detail-wrapper">
          {browseHeader}
          <ConnectorsDetail
            connector={selected}
            onToggleStatus={() => {
              if (selected.status === "connected") {
                setDialog({ kind: "confirm-disconnect", connectorId: selected.id });
              } else {
                toggleStatus(selected.id);
              }
            }}
            onDelete={() => setDialog({ kind: "confirm-delete", connectorId: selected.id })}
          />
        </div>
      ) : (
        <div className="ct-empty-wrapper">
          {browseHeader}
          <div className="ct-empty-center">
            <EmptyState message={t("connectors.sidebar.empty")} />
          </div>
        </div>
      )}

      {dialog.kind === "browse" && (
        <McpBrowseModal catalog={catalog} configuredIds={configuredIds} onPick={handlePick} onClose={() => setDialog({ kind: "none" })} />
      )}
      {dialog.kind === "config" && (
        <McpConfigDialog
          connector={dialog.connector}
          onClose={() => {
            setDialog(dialog.returnTo === "browse" ? { kind: "browse" } : { kind: "none" });
          }}
          onValidated={async () => {
            await addConnector(dialog.connector.id);
            setSelectedId(dialog.connector.id);
            setDialog({ kind: "none" });
          }}
        />
      )}
      {dialog.kind === "oauth-pending" && (
        <McpOauthDialog
          connector={dialog.connector}
          onClose={() => {
            setDialog(dialog.returnTo === "browse" ? { kind: "browse" } : { kind: "none" });
          }}
          onConnected={() => {
            addConnector(dialog.connector.id).then(() => {
              setSelectedId(dialog.connector.id);
              setDialog({ kind: "none" });
            });
          }}
        />
      )}
      {dialog.kind === "confirm-add" && (
        <div className="wk-dialog-overlay" onClick={() => setDialog(dialog.returnTo === "browse" ? { kind: "browse" } : { kind: "none" })}>
          <div className="wk-dialog" onClick={(e) => e.stopPropagation()}>
            <h3>{t("connectors.config.addTitle", { name: dialog.connector.display_name })}</h3>
            <p className="ct-confirm-desc">{t("connectors.config.confirmAddDesc", { name: dialog.connector.display_name })}</p>
            <div className="wk-dialog-footer">
              <button type="button" className="wk-btn-secondary" onClick={() => setDialog(dialog.returnTo === "browse" ? { kind: "browse" } : { kind: "none" })}>{t("connectors.detail.cancel")}</button>
              <button type="button" className="wk-btn-primary" onClick={async () => {
                await addConnector(dialog.connector.id);
                setSelectedId(dialog.connector.id);
                setDialog({ kind: "none" });
              }}>{t("connectors.config.confirmAddBtn")}</button>
            </div>
          </div>
        </div>
      )}
      {dialog.kind === "confirm-disconnect" && (
        <div className="wk-dialog-overlay" onClick={() => setDialog({ kind: "none" })}>
          <div className="wk-dialog" onClick={(e) => e.stopPropagation()}>
            <h3>{t("connectors.detail.confirmDisconnectTitle", { name: configured.find((c) => c.id === dialog.connectorId)?.display_name })}</h3>
            <p className="ct-confirm-desc">{t("connectors.detail.confirmDisconnectDesc", { name: configured.find((c) => c.id === dialog.connectorId)?.display_name })}</p>
            <div className="wk-dialog-footer">
              <button type="button" className="wk-btn-secondary" onClick={() => setDialog({ kind: "none" })}>{t("connectors.detail.cancel")}</button>
              <button type="button" className="wk-btn-primary ct-btn-danger" onClick={handleDisconnect}>{t("connectors.detail.confirmDisconnectBtn")}</button>
            </div>
          </div>
        </div>
      )}
      {dialog.kind === "confirm-delete" && (
        <div className="wk-dialog-overlay" onClick={() => setDialog({ kind: "none" })}>
          <div className="wk-dialog" onClick={(e) => e.stopPropagation()}>
            <h3>{t("connectors.detail.confirmDeleteTitle")}</h3>
            <div className="wk-dialog-footer">
              <button type="button" className="wk-btn-secondary" onClick={() => setDialog({ kind: "none" })}>{t("connectors.detail.cancel")}</button>
              <button type="button" className="wk-btn-primary ct-btn-danger" onClick={handleDelete}>{t("connectors.detail.confirmDeleteBtn")}</button>
            </div>
          </div>
        </div>
      )}
    </>
  );

  return { list, detail };
}
