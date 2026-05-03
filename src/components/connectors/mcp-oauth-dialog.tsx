import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-shell";
import { McpIcon } from "@/lib/mcp-icons";
import type { McpConnectorSpec } from "@/types/mcp";
import "./mcp-oauth-dialog.css";

interface McpOauthDialogProps {
  connector: McpConnectorSpec;
  onClose: () => void;
  onConnected: () => void;
}

export function McpOauthDialog({ connector, onClose, onConnected: _onConnected }: McpOauthDialogProps) {
  const { t } = useTranslation();

  const launchBrowser = () => { open(connector.url); };

  useEffect(() => { launchBrowser(); }, []);

  return (
    <div className="wk-dialog-overlay" onClick={onClose}>
      <div className="wk-dialog mco-dialog" onClick={(e) => e.stopPropagation()}>
        <div className="mco-icons">
          <div className="mco-icon-box">
            <McpIcon connectorId={connector.id} displayName={connector.display_name} size={40} />
          </div>
          <div className="mco-connector-line" />
          <div className="mco-icon-box">
            <span className="mco-app-icon">CL</span>
          </div>
        </div>

        <h3 className="mco-title">{t("connectors.oauth.title")}</h3>
        <p className="mco-message">{t("connectors.oauth.message")}</p>

        <button type="button" className="mco-retry-link" onClick={launchBrowser}>
          {t("connectors.oauth.retry")}
        </button>

        <div className="wk-dialog-footer mco-footer">
          <button type="button" className="wk-btn-secondary" onClick={onClose}>
            {t("connectors.oauth.cancel")}
          </button>
        </div>
      </div>
    </div>
  );
}
