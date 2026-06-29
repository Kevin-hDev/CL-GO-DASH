import type { ReactNode } from "react";
import { Spinner } from "@phosphor-icons/react";
import { CaretDown, CaretUp } from "@/components/ui/icons";
import { isFileTool } from "@/lib/tool-file-path";
import { FileIcon } from "@/components/file-preview/file-icon";
import { ToolIcon } from "./tool-icons";
import { ToolStatusIcon } from "./tool-status-icon";
import { useCollapsiblePresence } from "./use-collapsible-presence";

const FILE_TOOL_NAMES = new Set([
  "read_file", "write_file", "edit_file",
  "read_spreadsheet", "read_document", "read_image",
  "write_spreadsheet", "write_document", "process_image",
]);

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

function fileBaseName(path: string): string {
  const normalized = path.replace(/\\/g, "/");
  const parts = normalized.split("/").filter(Boolean);
  return parts.length > 0 ? parts[parts.length - 1] : path;
}

export function ToolItem({
  name, summary, icon, displayName, displaySummary, additions, deletions,
  done, isError, errorMessage, result, onFilePreview, children,
}: {
  name: string; summary: string; icon?: string; displayName?: string; displaySummary?: string;
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
  const isFileRow = FILE_TOOL_NAMES.has(name);
  const fileName = isFileRow ? fileBaseName(summary) : "";
  const openPreview = () => {
    if (!clickablePath || !onFilePreview) return;
    onFilePreview(summary);
  };

  return (
    <div>
      <div className="tb-row">
        {canToggle ? (
          <button className="tb-toggle" onClick={toggle}>
            <span className="tb-arrow tb-tool-arrow" aria-hidden="true">
              {isOpen ? <CaretUp size={13} weight="bold" /> : <CaretDown size={13} weight="bold" />}
            </span>
            {icon && <ToolIcon name={icon} size={14} className="tb-tool-icon" aria-hidden="true" />}
            <span className="tb-tool-verb">{shownName}</span>
          </button>
        ) : (
          <>
            {icon && <ToolIcon name={icon} size={14} className="tb-tool-icon" aria-hidden="true" />}
            <span className="tb-tool-verb">{shownName}</span>
          </>
        )}
        {isFileRow && fileName && (
          <FileIcon name={fileName} size={14} />
        )}
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
        {additions != null && deletions != null && (
          <span className="tb-change-stats">
            <span className="tb-change-add">+{additions}</span>
            <span className="tb-change-del">-{deletions}</span>
          </span>
        )}
        {!done && <Spinner size={14} className="tb-spinner" />}
        {done && (
          <ToolStatusIcon
            status={isError ? "error" : "success"}
            size={14}
            message={isError ? errorMessage : undefined}
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
