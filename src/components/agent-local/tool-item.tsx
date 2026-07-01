import type { ReactNode } from "react";
import { Spinner } from "@/components/ui/icons";
import { CaretDown, CaretUp } from "@/components/ui/icons";
import { isFileTool } from "@/lib/tool-file-path";
import { FileIcon } from "@/components/file-preview/file-icon";
import { ToolIcon } from "./tool-icons";
import { ToolStatusIcon } from "./tool-status-icon";
import { useCollapsiblePresence } from "./use-collapsible-presence";

const RESULT_PREVIEW_TOOLS = new Set([
  "bash", "grep", "glob", "read_file", "list_dir",
  "read_spreadsheet", "read_document", "read_image",
  "web_search", "web_fetch", "forecast", "forecast_read",
]);

function hasPreviewContent(children: ReactNode): boolean {
  if (!children) return false;
  if (Array.isArray(children)) return children.some((c) => !!c);
  return true;
}

export function ToolItem({
  name, summary, icon, displayName, displaySummary, dir, fileName,
  additions, deletions, done, isError, errorMessage, result, onFilePreview, children,
}: {
  name: string; summary: string; icon?: string; displayName?: string; displaySummary?: string;
  dir?: string; fileName?: string;
  additions?: number; deletions?: number; done: boolean; isError?: boolean; errorMessage?: string;
  result?: string; onFilePreview?: (path: string) => void; children?: ReactNode;
}) {
  const hasPreview = hasPreviewContent(children);
  const hasResult = !!result && !isError && !hasPreview && RESULT_PREVIEW_TOOLS.has(name);
  const canToggle = hasPreview || hasResult;
  const showCommandPreview = name === "bash" && hasResult;
  const { open: isOpen, mounted, toggle, onTransitionEnd } = useCollapsiblePresence();
  const clickablePath = isFileTool(name) && summary.trim().length > 0 && !!onFilePreview;
  const shownName = displayName ?? name;
  const shownSummary = displaySummary ?? summary;
  const hasFilePath = !!dir || !!fileName;
  const openPreview = () => {
    if (!clickablePath || !onFilePreview) return;
    onFilePreview(summary);
  };

  const labelButton = canToggle ? (
    <button className="tb-toggle" onClick={toggle}>
      <span className="tb-arrow tb-tool-arrow" aria-hidden="true">
        {isOpen ? <CaretUp size="var(--icon-sm)" weight="bold" /> : <CaretDown size="var(--icon-sm)" weight="bold" />}
      </span>
      {icon && <ToolIcon name={icon} size="var(--icon-sm)" className="tb-tool-icon" aria-hidden="true" />}
      <span className="tb-tool-verb">{shownName}</span>
    </button>
  ) : (
    <>
      {icon && <ToolIcon name={icon} size="var(--icon-sm)" className="tb-tool-icon" aria-hidden="true" />}
      <span className="tb-tool-verb">{shownName}</span>
    </>
  );

  // Cas fichier : nom + icône + stats collés à droite, dossiers tronqués à gauche
  const fileContent = hasFilePath ? (
    <>
      {dir && <span className="tb-item-dirs">{dir}</span>}
      <span className="tb-item-name">
        {fileName && <FileIcon name={fileName} size="var(--icon-sm)" />}
        <span className="tb-item-name-text">{fileName ?? shownSummary}</span>
      </span>
      {additions != null && deletions != null && (
        <span className="tb-change-stats">
          <span className="tb-change-add">+{additions}</span>
          <span className="tb-change-del">-{deletions}</span>
        </span>
      )}
    </>
  ) : null;

  // Cas non-fichier : résumé simple tronquable
  const summaryContent = !hasFilePath ? (
    <span
      className="tb-item-summary"
      role={clickablePath ? "button" : undefined}
      tabIndex={clickablePath ? 0 : undefined}
      onClick={(e) => { if (clickablePath) { e.stopPropagation(); openPreview(); } }}
      onKeyDown={(e) => {
        if (!clickablePath) return;
        if (e.key.startsWith("Ent") || e.key.startsWith(" ")) { e.preventDefault(); openPreview(); }
      }}
    >{shownSummary}</span>
  ) : null;

  return (
    <div>
      <div className="tb-row">
        {labelButton}
        {fileContent}
        {summaryContent}
        {!done && <Spinner size="var(--icon-sm)" className="tb-spinner" />}
        {done && isError && (
          <ToolStatusIcon
            size="var(--icon-sm)"
            message={errorMessage}
          />
        )}
      </div>
      {canToggle && (
        <div className={`tb-accordion${isOpen ? " tb-open" : ""}`} onTransitionEnd={onTransitionEnd}>
          {mounted && (
            <div className="tb-accordion-inner">
              {showCommandPreview && <div className="tb-command-preview">{summary}</div>}
              {hasPreview && children}
              {hasResult && <div className="tb-result-preview">{result}</div>}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
