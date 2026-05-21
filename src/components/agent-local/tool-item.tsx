import { useCallback, useRef, useState, type ReactNode } from "react";
import { Spinner } from "@phosphor-icons/react";
import { Copy, Check } from "@/components/ui/icons";
import { isFileTool } from "@/lib/tool-file-path";

const TOOL_COLORS: Record<string, string> = {
  bash: "var(--tool-bash)",
  glob: "var(--tool-search)", grep: "var(--tool-search)", list_dir: "var(--tool-search)",
  read_file: "var(--tool-read)", read_spreadsheet: "var(--tool-read)",
  read_document: "var(--tool-read)", read_image: "var(--tool-read)",
  write_file: "var(--tool-write)", write_spreadsheet: "var(--tool-write)",
  write_document: "var(--tool-write)",
  edit_file: "var(--tool-edit)", process_image: "var(--tool-edit)",
  web_search: "var(--tool-web)", web_fetch: "var(--tool-web)",
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
  display: "flex", alignItems: "baseline", gap: 8,
  fontSize: "11px", fontFamily: "var(--font-mono, monospace)", lineHeight: 1.6,
};

const HOVER_DELAY = 700;

function hasPreviewContent(children: ReactNode): boolean {
  if (!children) return false;
  if (Array.isArray(children)) return children.some((c) => !!c);
  return true;
}

function ErrorCross({ message }: { message?: string }) {
  const [visible, setVisible] = useState(false);
  const [copied, setCopied] = useState(false);
  const timerRef = useRef<ReturnType<typeof setTimeout>>(undefined);

  const enter = useCallback(() => {
    clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => setVisible(true), HOVER_DELAY);
  }, []);

  const leave = useCallback(() => {
    clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => { setVisible(false); setCopied(false); }, 100);
  }, []);

  const tooltipEnter = useCallback(() => {
    clearTimeout(timerRef.current);
  }, []);

  const copy = useCallback(() => {
    if (message) void navigator.clipboard.writeText(message).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  }, [message]);

  return (
    <span className="tb-error-anchor" onMouseEnter={enter} onMouseLeave={leave}>
      <span style={{ color: "var(--signal-error)", fontSize: "10px" }}>x</span>
      {visible && message && (
        <div className="tb-error-tooltip" onMouseEnter={tooltipEnter} onMouseLeave={leave}>
          <span className="tb-error-tooltip-text">{message}</span>
          <button type="button" className="tb-error-tooltip-copy" onClick={copy}>
            {copied ? <Check size={12} weight="bold" /> : <Copy size={12} />}
          </button>
        </div>
      )}
    </span>
  );
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
  const [isOpen, setIsOpen] = useState(false);
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
      <div style={ROW_STYLE}>
        {canToggle ? (
          <button className="tb-toggle" onClick={() => setIsOpen(!isOpen)}>
            <span className="tb-arrow">{isOpen ? "v" : ">"}</span>
            <span style={{ color: TOOL_COLORS[name] ?? "var(--ink-muted)", fontWeight: 600 }}>{shownName}</span>
          </button>
        ) : (
          <span style={{ color: TOOL_COLORS[name] ?? "var(--ink-muted)", fontWeight: 600, flexShrink: 0, minWidth: 70 }}>{shownName}</span>
        )}
        <span
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
        <div className={`tb-accordion${isOpen ? " tb-open" : ""}`}>
          {isOpen && (
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
