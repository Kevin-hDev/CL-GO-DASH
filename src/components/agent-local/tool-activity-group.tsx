import { useMemo, useState } from "react";
import { Spinner } from "@phosphor-icons/react";
import { CaretDown, CaretUp, Check } from "@/components/ui/icons";
import type { TFunction } from "i18next";
import { useTranslation } from "react-i18next";
import type { ToolActivityGroup, ToolActivityCounts } from "@/lib/tool-activity-summary";
import type { RenderableTool } from "./tool-detail-row";
import { ToolDetailRow } from "./tool-detail-row";

const COUNT_ORDER: Array<keyof ToolActivityCounts> = [
  "files",
  "searches",
  "lists",
  "writes",
  "edits",
  "commands",
  "webSearches",
  "webFetches",
  "gitActions",
  "otherActions",
];

function joinParts(parts: string[], language: string): string {
  if (parts.length <= 1) return parts[0] ?? "";
  try {
    return new Intl.ListFormat(language, { style: "long", type: "conjunction" }).format(parts);
  } catch {
    return parts.join(", ");
  }
}

function summaryText(
  group: ToolActivityGroup<RenderableTool>,
  t: TFunction,
  language: string,
): string {
  const parts = COUNT_ORDER
    .filter((key) => group.counts[key] > 0)
    .map((key) => t(`agentLocal.toolActivity.counts.${key}`, { count: group.counts[key] }));
  const label = t(`agentLocal.toolActivity.groups.${group.kind}`);
  const details = joinParts(parts, language);
  return details ? t("agentLocal.toolActivity.summary", { group: label, details }) : label;
}

function ToolActivityGroupRow({
  group,
  onFilePreview,
  projectPath,
}: {
  group: ToolActivityGroup<RenderableTool>;
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  const { t, i18n } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);
  const title = useMemo(() => summaryText(group, t, i18n.language), [group, t, i18n.language]);

  return (
    <div className={`tb-group tb-group-${group.kind}`}>
      <button
        type="button"
        className="tb-group-toggle"
        aria-expanded={isOpen}
        aria-label={t("agentLocal.toolActivity.toggleDetails")}
        onClick={() => setIsOpen((open) => !open)}
      >
        <span className="tb-arrow tb-group-arrow" aria-hidden="true">
          {isOpen ? <CaretUp size={14} weight="bold" /> : <CaretDown size={14} weight="bold" />}
        </span>
        <span className="tb-group-title">
          {title}
          {group.isPending && (
            <span className="tb-group-progress"> {t("agentLocal.toolActivity.inProgress")}</span>
          )}
        </span>
        <span className="tb-group-state" aria-hidden="true">
          {group.isPending && <Spinner size={12} className="tb-spinner" />}
          {!group.isPending && group.hasError && <span className="tb-group-error">x</span>}
          {!group.isPending && !group.hasError && <Check size={12} />}
        </span>
      </button>
      <div className={`tb-group-accordion${isOpen ? " tb-open" : ""}`}>
        {isOpen && (
          <div className="tb-group-details">
            {group.tools.map((tool, index) => (
              <ToolDetailRow
                key={`${tool.name}-${index}-${tool.summary}`}
                tool={tool}
                previousTools={group.tools.slice(0, index)}
                onFilePreview={onFilePreview}
                projectPath={projectPath}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export function ToolActivityGroupList({
  groups,
  onFilePreview,
  projectPath,
}: {
  groups: Array<ToolActivityGroup<RenderableTool>>;
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}) {
  return (
    <div className="tb-group-list">
      {groups.map((group) => (
        <ToolActivityGroupRow
          key={group.kind}
          group={group}
          onFilePreview={onFilePreview}
          projectPath={projectPath}
        />
      ))}
    </div>
  );
}
