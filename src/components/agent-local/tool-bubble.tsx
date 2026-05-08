import { useState } from "react";
import { Spinner } from "@phosphor-icons/react";
import type { ToolActivity } from "@/hooks/agent-chat-utils";
import type { ToolActivityRecord } from "@/types/agent";
import { isFileTool } from "@/lib/tool-file-path";
import { ContentPreview, DiffPreview, WebResultsPreview } from "./tool-previews";
import { ReadSpreadsheetPreview, WriteSpreadsheetPreview, DocumentResultPreview, WriteDocumentPreview } from "./tool-office-previews";
import "./tool-bubble.css";

const TOOL_COLORS: Record<string, string> = {
  bash: "var(--tool-bash)",
  glob: "var(--tool-search)", grep: "var(--tool-search)", list_dir: "var(--tool-search)",
  read_file: "var(--tool-read)", read_spreadsheet: "var(--tool-read)",
  read_document: "var(--tool-read)", read_image: "var(--tool-read)",
  write_file: "var(--tool-write)", write_spreadsheet: "var(--tool-write)",
  write_document: "var(--tool-write)",
  edit_file: "var(--tool-edit)", process_image: "var(--tool-edit)",
  web_search: "var(--tool-web)", web_fetch: "var(--tool-web)",
  create_branch: "var(--tool-git)", checkout_branch: "var(--tool-git)",
};

const CLOSED_BY_DEFAULT = new Set([
  "bash", "grep", "glob", "read_file", "list_dir",
  "read_spreadsheet", "read_document", "read_image",
  "web_search", "web_fetch",
]);

const ROW_STYLE = {
  display: "flex", alignItems: "baseline", gap: 8,
  fontSize: "11px", fontFamily: "var(--font-mono, monospace)", lineHeight: 1.6,
};

function parseLineFromResult(result?: string): number | undefined {
  if (!result) return undefined;
  const match = /\(ligne (\d+)\)/.exec(result);
  return match ? Number(match[1]) : undefined;
}

function str(v: unknown, fallback = ""): string {
  return typeof v === "string" ? v : fallback;
}

function toolSummary(t: ToolActivity): string {
  const a = t.args;
  if (t.name === "bash") return str(a.command);
  if (t.name === "grep") return str(a.pattern);
  if (t.name === "glob") return str(a.pattern);
  if (t.name === "read_file" || t.name === "write_file") return str(a.path);
  if (t.name === "edit_file") return str(a.path);
  if (t.name === "list_dir") return str(a.path, ".");
  if (t.name === "web_search") return str(a.query);
  if (t.name === "web_fetch") return str(a.url);
  if (t.name === "create_branch") return str(a.branch_name);
  if (t.name === "checkout_branch") return str(a.branch_name);
  if (t.name === "read_spreadsheet") return str(a.path);
  if (t.name === "read_document") return str(a.path);
  if (t.name === "read_image") return str(a.path);
  if (t.name === "write_spreadsheet") return str(a.path);
  if (t.name === "write_document") return str(a.path);
  if (t.name === "process_image") return str(a.input_path);
  return JSON.stringify(a).slice(0, 80);
}

export function ToolBubble({
  tools, onFilePreview,
}: { tools: ToolActivity[]; onFilePreview?: (path: string) => void }) {
  if (tools.length === 0) return null;
  return (
    <div className="chat-bubble">
      <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
        {tools.map((t, i) => {
          const skipWrite = t.name === "write_file" && i > 0
            && tools.slice(0, i).some((p) => p.name === "edit_file" && String(p.args.path) === String(t.args.path));
          return (
            <ToolItem key={i} name={t.name} summary={toolSummary(t)} done={!!t.result} isError={t.isError} errorMessage={t.isError ? t.result : undefined} result={t.result} onFilePreview={onFilePreview}>
              {t.name === "write_file" && !skipWrite && typeof t.args.content === "string" && <ContentPreview content={t.args.content} path={toolSummary(t)} />}
              {t.name === "edit_file" && typeof t.args.old_string === "string" && <DiffPreview oldText={t.args.old_string} newText={str(t.args.new_string)} path={toolSummary(t)} startLine={parseLineFromResult(t.result)} />}
              {(t.name === "web_search" || t.name === "web_fetch") && t.result && <WebResultsPreview content={t.result} isSearch={t.name === "web_search"} />}
              {t.name === "read_spreadsheet" && t.result && !t.isError && <ReadSpreadsheetPreview result={t.result} />}
              {t.name === "read_document" && t.result && !t.isError && <DocumentResultPreview result={t.result} />}
              {t.name === "write_spreadsheet" && t.result && !t.isError && <WriteSpreadsheetPreview operations={t.args.operations} />}
              {t.name === "write_document" && t.result && !t.isError && <WriteDocumentPreview content={t.args.content} />}
            </ToolItem>
          );
        })}
      </div>
    </div>
  );
}

export function SavedToolBubble({
  tools, onFilePreview,
}: { tools: ToolActivityRecord[]; onFilePreview?: (path: string) => void }) {
  if (tools.length === 0) return null;
  return (
    <div className="chat-bubble">
      <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
        {tools.map((t, i) => {
          const skipWrite = t.name === "write_file" && i > 0
            && tools.slice(0, i).some((p) => p.name === "edit_file" && p.summary === t.summary);
          return (
            <ToolItem key={i} name={t.name} summary={t.summary} done={t.result != null || t.is_error != null} isError={t.is_error} errorMessage={t.is_error ? t.result : undefined} result={t.result} onFilePreview={onFilePreview}>
              {t.name === "write_file" && t.content && !skipWrite && <ContentPreview content={t.content} path={t.summary} />}
              {t.old_text != null && t.new_text != null && <DiffPreview oldText={t.old_text} newText={t.new_text} path={t.summary} startLine={t.start_line} />}
              {(t.name === "web_search" || t.name === "web_fetch") && t.result && <WebResultsPreview content={t.result} isSearch={t.name === "web_search"} />}
              {t.name === "read_spreadsheet" && t.result && !t.is_error && <ReadSpreadsheetPreview result={t.result} />}
              {t.name === "read_document" && t.result && !t.is_error && <DocumentResultPreview result={t.result} />}
              {t.name === "write_spreadsheet" && t.result && !t.is_error && t.content && <WriteSpreadsheetPreview operations={t.content} />}
              {t.name === "write_document" && t.result && !t.is_error && t.content && <WriteDocumentPreview content={t.content} />}
            </ToolItem>
          );
        })}
      </div>
    </div>
  );
}

const ERROR_STYLE = {
  color: "var(--signal-error)",
  fontSize: "10px",
  fontFamily: "var(--font-mono, monospace)",
  lineHeight: 1.4,
  marginTop: 2,
  marginLeft: 24,
  opacity: 0.85,
  wordBreak: "break-word" as const,
};

function hasPreviewContent(children: React.ReactNode): boolean {
  if (!children) return false;
  if (Array.isArray(children)) return children.some((c) => !!c);
  return true;
}

function ToolItem({ name, summary, done, isError, errorMessage, result, onFilePreview, children }: {
  name: string; summary: string; done: boolean; isError?: boolean; errorMessage?: string;
  result?: string; onFilePreview?: (path: string) => void; children?: React.ReactNode;
}) {
  const hasPreview = hasPreviewContent(children);
  const hasResult = !!result && !isError && !hasPreview && CLOSED_BY_DEFAULT.has(name);
  const canToggle = hasPreview || hasResult;
  const [isOpen, setIsOpen] = useState(!CLOSED_BY_DEFAULT.has(name));
  const clickablePath = isFileTool(name) && summary.trim().length > 0 && !!onFilePreview;

  return (
    <div>
      <div style={ROW_STYLE}>
        {canToggle ? (
          <button className="tb-toggle" onClick={() => setIsOpen(!isOpen)}>
            <span className="tb-arrow">{isOpen ? "▾" : "▸"}</span>
            <span style={{ color: TOOL_COLORS[name] ?? "var(--ink-muted)", fontWeight: 600 }}>{name}</span>
          </button>
        ) : (
          <span style={{ color: TOOL_COLORS[name] ?? "var(--ink-muted)", fontWeight: 600, flexShrink: 0, minWidth: 70 }}>{name}</span>
        )}
        <span
          role={clickablePath ? "button" : undefined}
          tabIndex={clickablePath ? 0 : undefined}
          style={{
            color: clickablePath ? "var(--ink)" : "var(--ink-muted)",
            wordBreak: "break-all", flex: 1,
            cursor: clickablePath ? "pointer" : "default",
            textDecoration: clickablePath ? "underline" : "none",
            textDecorationColor: "var(--edge)",
          }}
          onClick={(e) => { if (clickablePath) { e.stopPropagation(); onFilePreview(summary); } }}
          onKeyDown={(e) => {
            if (!clickablePath) return;
            if (e.key.startsWith("Ent") || e.key.startsWith(" ")) { e.preventDefault(); onFilePreview(summary); }
          }}
        >{summary}</span>
        {!done && <Spinner size={12} style={{ color: "var(--ink-faint)", animation: "spin 1s linear infinite", flexShrink: 0 }} />}
        {done && <span style={{ color: isError ? "var(--signal-error)" : "var(--signal-ok)", flexShrink: 0, fontSize: "10px" }}>{isError ? "✗" : "✓"}</span>}
      </div>
      {isError && errorMessage && <div style={ERROR_STYLE}>{errorMessage}</div>}
      {canToggle && (
        <div className={`tb-accordion${isOpen ? " tb-open" : ""}`}>
          <div className="tb-accordion-inner">
            {hasPreview && children}
            {hasResult && <div className="tb-result-preview">{result}</div>}
          </div>
        </div>
      )}
    </div>
  );
}
