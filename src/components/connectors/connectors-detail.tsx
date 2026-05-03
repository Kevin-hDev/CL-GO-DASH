import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-shell";
import { Trash, ArrowSquareOut } from "@/components/ui/icons";
import { McpIcon } from "@/lib/mcp-icons";
import { SettingsCard } from "@/components/settings/settings-card";
import { getMcpDescription } from "@/types/mcp";
import type { ConfiguredMcpFull } from "@/types/mcp";
import "./connectors-detail.css";

interface ConnectorsDetailProps {
  connector: ConfiguredMcpFull;
  onToggleStatus: () => void;
  onDelete: () => void;
}

export function ConnectorsDetail({ connector, onToggleStatus, onDelete }: ConnectorsDetailProps) {
  const { t, i18n } = useTranslation();
  const isConnected = connector.status === "connected";

  return (
    <div className="ctd-scroll">
      <div className="ctd-inner">
        <div className="ctd-header">
          <div className="ctd-info">
            <McpIcon connectorId={connector.id} displayName={connector.display_name} size={36} />
            <div>
              <h2 className="ctd-name">{connector.display_name}</h2>
              <span className="ctd-author-small">{connector.author}</span>
            </div>
          </div>
          <div className="ctd-actions">
            <button
              type="button"
              className={`ctd-status-btn ${isConnected ? "connected" : "disconnected"}`}
              onClick={onToggleStatus}
            >
              <span className="ctd-status-dot" />
              {t(isConnected ? "connectors.detail.connected" : "connectors.detail.disconnected")}
            </button>
            <button type="button" className="ak-icon-btn" onClick={onDelete} title={t("connectors.detail.confirmDeleteBtn")}>
              <Trash size={16} />
            </button>
          </div>
        </div>

        <SettingsCard>
          <div className="ctd-section">
            <div className="ctd-section-label">{t("connectors.detail.description")}</div>
            <div className="ctd-section-value">{getMcpDescription(connector, i18n.language)}</div>
          </div>
        </SettingsCard>

        <SettingsCard>
          <div className="ctd-section">
            <div className="ctd-section-label">{t("connectors.detail.tools")}</div>
            <div className="ctd-tools">
              {connector.tools.map((tool) => (
                <span key={tool} className="ctd-tool-badge">{tool}</span>
              ))}
            </div>
          </div>
        </SettingsCard>

        <SettingsCard>
          <div className="ctd-row ctd-row-border">
            <span className="ctd-row-label">{t("connectors.detail.author")}</span>
            <span className="ctd-row-value">{connector.author}</span>
          </div>
          <div className="ctd-row">
            <span className="ctd-row-label">{t("connectors.detail.website")}</span>
            <button
              type="button"
              className="ctd-link"
              onClick={() => open(connector.url)}
            >
              {t("connectors.detail.openSite")} <ArrowSquareOut size={12} />
            </button>
          </div>
        </SettingsCard>
      </div>
    </div>
  );
}
