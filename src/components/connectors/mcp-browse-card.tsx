import { useTranslation } from "react-i18next";
import { open } from "@tauri-apps/plugin-shell";
import { Plus, Check, ArrowSquareOut } from "@/components/ui/icons";
import { McpIcon } from "@/lib/mcp-icons";
import { getMcpDescription } from "@/types/mcp";
import type { McpConnectorSpec } from "@/types/mcp";
import "./mcp-browse-card.css";

interface McpBrowseCardProps {
  connector: McpConnectorSpec;
  configured: boolean;
  onAdd: () => void;
}

export function McpBrowseCard({ connector, configured, onAdd }: McpBrowseCardProps) {
  const { i18n } = useTranslation();

  const handleLinkClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    open(connector.url);
  };

  return (
    <div className={`mcbc-card ${configured ? "configured" : ""}`}>
      <McpIcon connectorId={connector.id} displayName={connector.display_name} size={40} />
      <div className="mcbc-body">
        <div className="mcbc-name">{connector.display_name}</div>
        <div className="mcbc-desc">{getMcpDescription(connector, i18n.language)}</div>
        <div className="mcbc-meta">
          <span className="mcbc-cat">{connector.category.toUpperCase()}</span>
          <span className="mcbc-author">{connector.author}</span>
          <button type="button" className="mcbc-link" onClick={handleLinkClick} title={connector.url}>
            <ArrowSquareOut size={12} />
          </button>
        </div>
      </div>
      <button
        type="button"
        className={`mcbc-action ${configured ? "done" : ""}`}
        onClick={configured ? undefined : onAdd}
        disabled={configured}
      >
        {configured ? <Check size={16} weight="bold" /> : <Plus size={16} weight="bold" />}
      </button>
    </div>
  );
}
