import { TerminalWindow, Spinner } from "@phosphor-icons/react";
import type { ToolActivity } from "@/hooks/agent-chat-utils";
import type { ToolActivityRecord } from "@/types/agent";

const TOOL_COLORS: Record<string, string> = {
  shell: "#f97316",
  read_file: "#3db86a",
  write_file: "#e2b842",
  edit_file: "#e2b842",
  list_dir: "#4a8fe2",
  web_search: "#9b7fff",
  web_fetch: "#9b7fff",
};

const BUBBLE_STYLE = {
  width: "85%",
  background: "#0d0d0f",
  border: "1px solid rgba(255,255,255,0.06)",
  borderRadius: "var(--radius-md, 8px)",
  padding: "10px 14px",
  alignSelf: "center" as const,
  margin: "6px auto",
};

const HEADER_STYLE = {
  display: "flex", alignItems: "center", gap: 6,
  marginBottom: 8, opacity: 0.5,
  fontSize: "11px", color: "#888",
  textTransform: "uppercase" as const, letterSpacing: "0.5px",
};

const ROW_STYLE = {
  display: "flex", alignItems: "baseline", gap: 8,
  fontSize: "11px", fontFamily: "var(--font-mono, monospace)",
  lineHeight: 1.6,
};

function toolSummary(t: ToolActivity): string {
  const a = t.args;
  if (t.name === "shell") return String(a.command ?? "");
  if (t.name === "read_file" || t.name === "write_file") return String(a.path ?? "");
  if (t.name === "edit_file") return String(a.path ?? "");
  if (t.name === "list_dir") return String(a.path ?? ".");
  if (t.name === "web_search") return String(a.query ?? "");
  if (t.name === "web_fetch") return String(a.url ?? "");
  return JSON.stringify(a).slice(0, 80);
}

// Streaming tool bubble (pendant que GEMMA travaille)
export function ToolBubble({ tools }: { tools: ToolActivity[] }) {
  if (tools.length === 0) return null;
  return (
    <div style={BUBBLE_STYLE}>
      <div style={HEADER_STYLE}>
        <TerminalWindow size={12} weight="bold" />
        Tools
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
        {tools.map((t, i) => {
          // Masquer le contenu write_file si un edit_file sur le même path le précède
          const writeAfterEdit = t.name === "write_file" && i > 0
            && tools.slice(0, i).some((prev) =>
              prev.name === "edit_file" && String(prev.args.path) === String(t.args.path));

          return (
            <div key={i}>
              <div style={ROW_STYLE}>
                <span style={{ color: TOOL_COLORS[t.name] ?? "#888", fontWeight: 600, flexShrink: 0, minWidth: 70 }}>
                  {t.name}
                </span>
                <span style={{ color: "#999", wordBreak: "break-all", flex: 1 }}>
                  {toolSummary(t)}
                </span>
                {!t.result && (
                  <Spinner size={12} style={{ color: "#666", animation: "spin 1s linear infinite", flexShrink: 0 }} />
                )}
                {t.result && (
                  <span style={{ color: t.isError ? "#f87171" : "#4ade80", flexShrink: 0, fontSize: "10px" }}>
                    {t.isError ? "✗" : "✓"}
                  </span>
                )}
              </div>
              {t.name === "write_file" && !writeAfterEdit && typeof t.args.content === "string" && (
                <ContentPreview content={t.args.content} />
              )}
              {t.name === "edit_file" && typeof t.args.old_string === "string" && (
                <DiffPreview oldText={t.args.old_string} newText={String(t.args.new_string ?? "")} />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

// Saved tool bubble (persisté dans le message assistant)
export function SavedToolBubble({ tools }: { tools: ToolActivityRecord[] }) {
  if (tools.length === 0) return null;
  return (
    <div style={BUBBLE_STYLE}>
      <div style={HEADER_STYLE}>
        <TerminalWindow size={12} weight="bold" />
        Tools
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
        {tools.map((t, i) => {
          const writeAfterEdit = t.name === "write_file" && i > 0
            && tools.slice(0, i).some((prev) =>
              prev.name === "edit_file" && prev.summary === t.summary);

          return (
            <div key={i}>
              <div style={ROW_STYLE}>
                <span style={{ color: TOOL_COLORS[t.name] ?? "#888", fontWeight: 600, flexShrink: 0, minWidth: 70 }}>
                  {t.name}
                </span>
                <span style={{ color: "#999", wordBreak: "break-all", flex: 1 }}>
                  {t.summary}
                </span>
                {t.is_error != null && (
                  <span style={{ color: t.is_error ? "#f87171" : "#4ade80", flexShrink: 0, fontSize: "10px" }}>
                    {t.is_error ? "✗" : "✓"}
                  </span>
                )}
              </div>
              {t.content && !writeAfterEdit && (
                <ContentPreview content={t.content} />
              )}
              {t.old_text != null && t.new_text != null && (
                <DiffPreview oldText={t.old_text} newText={t.new_text} />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

const LINE_STYLE: React.CSSProperties = {
  display: "flex",
  fontFamily: "var(--font-mono, monospace)",
  fontSize: "var(--text-xs, 11px)",
  lineHeight: 1.7,
};

const NUM_STYLE: React.CSSProperties = {
  width: 32, textAlign: "right", paddingRight: 8,
  color: "#555", userSelect: "none", flexShrink: 0,
};

const PREFIX_STYLE: React.CSSProperties = {
  width: 16, textAlign: "center", flexShrink: 0, userSelect: "none",
};

const CODE_STYLE: React.CSSProperties = {
  flex: 1, whiteSpace: "pre-wrap", wordBreak: "break-all",
  paddingRight: 8,
};

function ContentPreview({ content }: { content: string }) {
  const lines = content.split("\n");
  return (
    <div style={{
      marginTop: 6, borderRadius: 4, overflow: "hidden",
      border: "1px solid rgba(255,255,255,0.06)",
    }}>
      {lines.map((line, i) => (
        <div key={i} style={{
          ...LINE_STYLE,
          background: "rgba(34, 197, 94, 0.15)",
        }}>
          <span style={NUM_STYLE}>{i + 1}</span>
          <span style={{ ...PREFIX_STYLE, color: "#22c55e" }}>+</span>
          <span style={{ ...CODE_STYLE, color: "#86efac" }}>{line}</span>
        </div>
      ))}
    </div>
  );
}

function DiffPreview({ oldText, newText }: { oldText: string; newText: string }) {
  const oldLines = oldText.split("\n");
  const newLines = newText.split("\n");
  return (
    <div style={{
      marginTop: 6, borderRadius: 4, overflow: "hidden",
      border: "1px solid rgba(255,255,255,0.06)",
    }}>
      {oldLines.map((line, i) => (
        <div key={`old-${i}`} style={{
          ...LINE_STYLE,
          background: "rgba(220, 38, 38, 0.15)",
        }}>
          <span style={NUM_STYLE}>{i + 1}</span>
          <span style={{ ...PREFIX_STYLE, color: "#ef4444" }}>-</span>
          <span style={{ ...CODE_STYLE, color: "#fca5a5" }}>{line}</span>
        </div>
      ))}
      {newLines.map((line, i) => (
        <div key={`new-${i}`} style={{
          ...LINE_STYLE,
          background: "rgba(34, 197, 94, 0.15)",
        }}>
          <span style={NUM_STYLE}>{i + 1}</span>
          <span style={{ ...PREFIX_STYLE, color: "#22c55e" }}>+</span>
          <span style={{ ...CODE_STYLE, color: "#86efac" }}>{line}</span>
        </div>
      ))}
    </div>
  );
}
