import { TerminalWindow, Spinner } from "@phosphor-icons/react";
import { useTranslation } from "react-i18next";
import type { ToolActivity } from "@/hooks/agent-chat-utils";
import type { ToolActivityRecord } from "@/types/agent";
import { isFileTool } from "@/lib/tool-file-path";
import { ContentPreview, DiffPreview, WebResultsPreview } from "./tool-previews";

const TOOL_COLORS: Record<string, string> = {
  bash: "var(--tool-bash)", read_file: "var(--tool-file-read)",
  write_file: "var(--tool-file-write)", edit_file: "var(--tool-file-write)",
  list_dir: "var(--tool-search)", grep: "var(--tool-search)", glob: "var(--tool-search)",
  web_search: "var(--tool-web)", web_fetch: "var(--tool-web)",
};

const BUBBLE_STYLE = {
  width: "100%", maxWidth: "720px", background: "var(--void)",
  border: "1px solid var(--edge)",
  borderRadius: "var(--radius-md, 8px)",
  padding: "10px 14px", alignSelf: "center" as const, margin: "6px auto",
};

const HEADER_STYLE = {
  display: "flex", alignItems: "center", gap: 6,
  marginBottom: 8, opacity: 0.5,
  fontSize: "11px", color: "var(--ink-muted)",
  textTransform: "uppercase" as const, letterSpacing: "0.5px",
};

const ROW_STYLE = {
  display: "flex", alignItems: "baseline", gap: 8,
  fontSize: "11px", fontFamily: "var(--font-mono, monospace)", lineHeight: 1.6,
};

function parseLineFromResult(result?: string): number | undefined {
  if (!result) return undefined;
  const match = /\(ligne (\d+)\)/.exec(result);
  return match ? Number(match[1]) : undefined;
}

function toolSummary(t: ToolActivity): string {
  const a = t.args;
  if (t.name === "bash") return String(a.command ?? "");
  if (t.name === "grep") return String(a.pattern ?? "");
  if (t.name === "glob") return String(a.pattern ?? "");
  if (t.name === "read_file" || t.name === "write_file") return String(a.path ?? "");
  if (t.name === "edit_file") return String(a.path ?? "");
  if (t.name === "list_dir") return String(a.path ?? ".");
  if (t.name === "web_search") return String(a.query ?? "");
  if (t.name === "web_fetch") return String(a.url ?? "");
  return JSON.stringify(a).slice(0, 80);
}

export function ToolBubble({
  tools,
  onFilePreview,
}: { tools: ToolActivity[]; onFilePreview?: (path: string) => void }) {
  const { t: tl } = useTranslation();
  if (tools.length === 0) return null;
  return (
    <div style={BUBBLE_STYLE}>
      <div style={HEADER_STYLE}><TerminalWindow size={12} weight="bold" /> {tl("agentLocal.tools")}</div>
      <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
        {tools.map((t, i) => {
          const skipWrite = t.name === "write_file" && i > 0
            && tools.slice(0, i).some((p) => p.name === "edit_file" && String(p.args.path) === String(t.args.path));
          return (
            <div key={i}>
              <ToolRow name={t.name} summary={toolSummary(t)} done={!!t.result} isError={t.isError} errorMessage={t.isError ? t.result : undefined} onFilePreview={onFilePreview} />
              {t.name === "write_file" && !skipWrite && typeof t.args.content === "string" && <ContentPreview content={t.args.content} path={toolSummary(t)} />}
              {t.name === "edit_file" && typeof t.args.old_string === "string" && <DiffPreview oldText={t.args.old_string} newText={String(t.args.new_string ?? "")} path={toolSummary(t)} startLine={parseLineFromResult(t.result)} />}
              {(t.name === "web_search" || t.name === "web_fetch") && t.result && <WebResultsPreview content={t.result} isSearch={t.name === "web_search"} />}
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function SavedToolBubble({
  tools,
  onFilePreview,
}: { tools: ToolActivityRecord[]; onFilePreview?: (path: string) => void }) {
  const { t: tl } = useTranslation();
  if (tools.length === 0) return null;
  return (
    <div style={BUBBLE_STYLE}>
      <div style={HEADER_STYLE}><TerminalWindow size={12} weight="bold" /> {tl("agentLocal.tools")}</div>
      <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
        {tools.map((t, i) => {
          const skipWrite = t.name === "write_file" && i > 0
            && tools.slice(0, i).some((p) => p.name === "edit_file" && p.summary === t.summary);
          return (
            <div key={i}>
              <ToolRow name={t.name} summary={t.summary} done={t.is_error != null} isError={t.is_error} errorMessage={t.is_error ? t.result : undefined} onFilePreview={onFilePreview} />
              {t.content && !skipWrite && <ContentPreview content={t.content} path={t.summary} />}
              {t.old_text != null && t.new_text != null && <DiffPreview oldText={t.old_text} newText={t.new_text} path={t.summary} startLine={t.start_line} />}
              {(t.name === "web_search" || t.name === "web_fetch") && t.result && <WebResultsPreview content={t.result} isSearch={t.name === "web_search"} />}
            </div>
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
  marginLeft: 78,
  opacity: 0.85,
  wordBreak: "break-word" as const,
};

function ToolRow({ name, summary, done, isError, errorMessage, onFilePreview }: {
  name: string; summary: string; done: boolean; isError?: boolean; errorMessage?: string; onFilePreview?: (path: string) => void;
}) {
  const clickable = isFileTool(name) && summary.trim().length > 0 && !!onFilePreview;
  return (
    <div>
      <div style={ROW_STYLE}>
        <span style={{ color: TOOL_COLORS[name] ?? "var(--ink-muted)", fontWeight: 600, flexShrink: 0, minWidth: 70 }}>{name}</span>
        <span
          role={clickable ? "button" : undefined}
          tabIndex={clickable ? 0 : undefined}
          style={{
            color: clickable ? "var(--ink)" : "var(--ink-muted)",
            wordBreak: "break-all",
            flex: 1,
            cursor: clickable ? "pointer" : "default",
            textDecoration: clickable ? "underline" : "none",
            textDecorationColor: "var(--edge)",
          }}
          onClick={() => { if (clickable) onFilePreview!(summary); }}
          onKeyDown={(e) => {
            if (!clickable) return;
            if (e.key.startsWith("Ent") || e.key.startsWith(" ")) {
              e.preventDefault();
              onFilePreview!(summary);
            }
          }}
        >{summary}</span>
        {!done && <Spinner size={12} style={{ color: "var(--ink-faint)", animation: "spin 1s linear infinite", flexShrink: 0 }} />}
        {done && <span style={{ color: isError ? "var(--signal-error)" : "var(--signal-ok)", flexShrink: 0, fontSize: "10px" }}>{isError ? "✗" : "✓"}</span>}
      </div>
      {isError && errorMessage && <div style={ERROR_STYLE}>{errorMessage}</div>}
    </div>
  );
}
