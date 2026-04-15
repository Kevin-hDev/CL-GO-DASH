import { TerminalWindow, Spinner } from "@phosphor-icons/react";
import type { ToolActivity } from "@/hooks/agent-chat-utils";
import type { ToolActivityRecord } from "@/types/agent";
import { ContentPreview, DiffPreview, WebResultsPreview } from "./tool-previews";

const TOOL_COLORS: Record<string, string> = {
  bash: "#f97316", read_file: "#3db86a",
  write_file: "#e2b842", edit_file: "#e2b842",
  list_dir: "#4a8fe2", grep: "#4a8fe2", glob: "#4a8fe2",
  web_search: "#9b7fff", web_fetch: "#9b7fff",
};

const BUBBLE_STYLE = {
  width: "85%", background: "#0d0d0f",
  border: "1px solid rgba(255,255,255,0.06)",
  borderRadius: "var(--radius-md, 8px)",
  padding: "10px 14px", alignSelf: "center" as const, margin: "6px auto",
};

const HEADER_STYLE = {
  display: "flex", alignItems: "center", gap: 6,
  marginBottom: 8, opacity: 0.5,
  fontSize: "11px", color: "#888",
  textTransform: "uppercase" as const, letterSpacing: "0.5px",
};

const ROW_STYLE = {
  display: "flex", alignItems: "baseline", gap: 8,
  fontSize: "11px", fontFamily: "var(--font-mono, monospace)", lineHeight: 1.6,
};

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

export function ToolBubble({ tools }: { tools: ToolActivity[] }) {
  if (tools.length === 0) return null;
  return (
    <div style={BUBBLE_STYLE}>
      <div style={HEADER_STYLE}><TerminalWindow size={12} weight="bold" /> Tools</div>
      <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
        {tools.map((t, i) => {
          const skipWrite = t.name === "write_file" && i > 0
            && tools.slice(0, i).some((p) => p.name === "edit_file" && String(p.args.path) === String(t.args.path));
          return (
            <div key={i}>
              <ToolRow name={t.name} summary={toolSummary(t)} done={!!t.result} isError={t.isError} />
              {t.name === "write_file" && !skipWrite && typeof t.args.content === "string" && <ContentPreview content={t.args.content} />}
              {t.name === "edit_file" && typeof t.args.old_string === "string" && <DiffPreview oldText={t.args.old_string} newText={String(t.args.new_string ?? "")} />}
              {(t.name === "web_search" || t.name === "web_fetch") && t.result && <WebResultsPreview content={t.result} isSearch={t.name === "web_search"} />}
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function SavedToolBubble({ tools }: { tools: ToolActivityRecord[] }) {
  if (tools.length === 0) return null;
  return (
    <div style={BUBBLE_STYLE}>
      <div style={HEADER_STYLE}><TerminalWindow size={12} weight="bold" /> Tools</div>
      <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
        {tools.map((t, i) => {
          const skipWrite = t.name === "write_file" && i > 0
            && tools.slice(0, i).some((p) => p.name === "edit_file" && p.summary === t.summary);
          return (
            <div key={i}>
              <ToolRow name={t.name} summary={t.summary} done={t.is_error != null} isError={t.is_error} />
              {t.content && !skipWrite && <ContentPreview content={t.content} />}
              {t.old_text != null && t.new_text != null && <DiffPreview oldText={t.old_text} newText={t.new_text} />}
              {(t.name === "web_search" || t.name === "web_fetch") && t.result && <WebResultsPreview content={t.result} isSearch={t.name === "web_search"} />}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function ToolRow({ name, summary, done, isError }: {
  name: string; summary: string; done: boolean; isError?: boolean;
}) {
  return (
    <div style={ROW_STYLE}>
      <span style={{ color: TOOL_COLORS[name] ?? "#888", fontWeight: 600, flexShrink: 0, minWidth: 70 }}>{name}</span>
      <span style={{ color: "#999", wordBreak: "break-all", flex: 1 }}>{summary}</span>
      {!done && <Spinner size={12} style={{ color: "#666", animation: "spin 1s linear infinite", flexShrink: 0 }} />}
      {done && <span style={{ color: isError ? "#f87171" : "#4ade80", flexShrink: 0, fontSize: "10px" }}>{isError ? "✗" : "✓"}</span>}
    </div>
  );
}
