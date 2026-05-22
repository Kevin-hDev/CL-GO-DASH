import type { ReactNode } from "react";
import { Spinner } from "@phosphor-icons/react";
import { CaretDown, CaretUp, Check } from "@/components/ui/icons";
import { isFileTool } from "@/lib/tool-file-path";
import { ErrorCross } from "./tool-error-tooltip";
import { useCollapsiblePresence } from "./use-collapsible-presence";

const TOOL_COLORS: Record<string, string> = {
  bash: "var(--tool-bash)",
  glob: "var(--tool-search)", grep: "var(--tool-search)", list_dir: "var(--tool-search)",
  read_file: "var(--tool-read)", read_spreadsheet: "var(--tool-read)",
  read_document: "var(--tool-read)", read_image: "var(--tool-read)",
  write_file: "var(--tool-edit)", write_spreadsheet: "var(--tool-edit)",
  write_document: "var(--tool-edit)",
  edit_file: "var(--tool-edit)", process_image: "var(--tool-edit)",
  web_search: "var(--ink)", web_fetch: "var(--ink)",
  create_branch: "var(--tool-bash)", checkout_branch: "var(--tool-bash)",
  forecast: "var(--tool-forecast)", forecast_analyze: "var(--tool-forecast)",
  forecast_read: "var(--tool-forecast)",
};

const RESULT_PREVIEW_TOOLS = new Set([
  "bash", "grep", "glob", "read_file", "list_dir",
  "read_spreadsheet", "read_document", "read_image",
  "web_search", "web_fetch", "forecast", "forecast_read",
]);

const ROW_STYLE = {
  display: "flex", alignItems: "center", gap: 8,
  fontSize: "11px", fontFamily: "var(--font-mono, monospace)", lineHeight: 1.6,
};

function hasPreviewContent(children: ReactNode): boolean {
  if (!children) return false;
  if (Array.isArray(children)) return children.some((c) => !!c);
  return true;
}

export function ToolItem({
  name, summary, displayName, displaySummary, additions, deletions,
  done, isError, errorMessage, result, onFilePreview, children,
}: {
  name: string; summary: string; displayName?: string; displaySummary?: string;
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
  const forceSingleLineSummary = name === "bash";
  const openPreview = () => {
    if (!clickablePath || !onFilePreview) return;
    onFilePreview(summary);
  };

  return (
    <div>
      <div className="tb-row" style={ROW_STYLE}>
        {canToggle ? (
          <button className="tb-toggle" onClick={toggle}>
            <span className="tb-arrow tb-tool-arrow" aria-hidden="true">
              {isOpen ? <CaretUp size={13} weight="bold" /> : <CaretDown size={13} weight="bold" />}
            </span>
            <span style={{ color: TOOL_COLORS[name] ?? "var(--ink-muted)", fontWeight: 600 }}>{shownName}</span>
          </button>
        ) : (
          <span style={{ color: TOOL_COLORS[name] ?? "var(--ink-muted)", fontWeight: 600, flexShrink: 0, minWidth: 70 }}>{shownName}</span>
        )}
        <span
          className="tb-item-summary"
          role={clickablePath ? "button" : undefined}
          tabIndex={clickablePath ? 0 : undefined}
          style={{
            color: clickablePath ? "var(--ink)" : "var(--ink-muted)",
            wordBreak: forceSingleLineSummary ? "normal" : "break-all",
            flex: 1,
            minWidth: 0,
            overflow: forceSingleLineSummary ? "hidden" : undefined,
            textOverflow: forceSingleLineSummary ? "ellipsis" : undefined,
            whiteSpace: forceSingleLineSummary ? "nowrap" : undefined,
            cursor: clickablePath ? "pointer" : "default",
            textDecoration: clickablePath ? "underline" : "none",
            textDecorationColor: "var(--edge)",
          }}
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
        {!done && <Spinner size={12} style={{ color: "var(--ink-faint)", animation: "spin 1s linear infinite", flexShrink: 0 }} />}
        {done && !isError && <Check size={12} style={{ color: "var(--signal-ok)", flexShrink: 0 }} />}
        {done && isError && <ErrorCross message={errorMessage} />}
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
