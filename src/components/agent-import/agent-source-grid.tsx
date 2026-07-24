import { useTranslation } from "react-i18next";
import { CaretRight, FolderOpen } from "@/components/ui/icons";
import type { AgentSourceSummary } from "@/types/agent-import";

interface AgentSourceGridProps {
  sources: AgentSourceSummary[];
  onSelect: (sourceId: string) => void;
}

export function AgentSourceGrid({ sources, onSelect }: AgentSourceGridProps) {
  const { t } = useTranslation();

  return (
    <div className="aim-grid">
      {sources.map((source) => {
        const activeCount = [
          ...source.documents,
          ...source.rules,
          ...source.skills,
        ].filter((item) => item.selected).length;
        return (
          <button
            key={source.id}
            type="button"
            className={`aim-source-card aim-status-${source.status}`}
            onClick={() => onSelect(source.id)}
          >
            <span className="aim-source-icon">
              <FolderOpen size="var(--icon-lg)" weight="duotone" />
            </span>
            <span className="aim-source-copy">
              <span className="aim-source-name">{source.displayName}</span>
              <span className="aim-source-status">
                {t(`agentImport.status.${source.status}`)}
              </span>
              <span className="aim-source-count">
                {t("agentImport.card.counts", {
                  documents: source.documents.length,
                  rules: source.rules.length,
                  skills: source.skills.length,
                })}
              </span>
              {source.configured && (
                <span className="aim-source-active">
                  {source.enabled
                    ? t("agentImport.card.activeCount", { count: activeCount })
                    : t("agentImport.card.disabled")}
                </span>
              )}
            </span>
            <CaretRight className="aim-source-caret" size="var(--icon-md)" />
          </button>
        );
      })}
    </div>
  );
}
