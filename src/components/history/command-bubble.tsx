import type { ToolCall } from "@/types/session";
import { TerminalWindow } from "@phosphor-icons/react";
import { DiffView } from "./diff-view";

interface CommandBubbleProps {
  tools: ToolCall[];
}

const TOOL_COLORS: Record<string, string> = {
  Bash: "#f97316",
  Read: "#3db86a",
  Write: "#e2b842",
  Edit: "#e2b842",
  Grep: "#4a8fe2",
  Glob: "#4a8fe2",
  Skill: "#9b7fff",
};

export function CommandBubble({ tools }: CommandBubbleProps) {
  return (
    <div style={{
      width: "85%",
      background: "#0d0d0f",
      border: "1px solid rgba(255,255,255,0.06)",
      borderRadius: "var(--radius-md)",
      padding: "10px 14px",
      alignSelf: "center",
    }}>
      <div style={{
        display: "flex", alignItems: "center", gap: 6,
        marginBottom: 8, opacity: 0.5,
        fontSize: "var(--text-xs)", color: "#888",
        textTransform: "uppercase", letterSpacing: "0.5px",
      }}>
        <TerminalWindow size={12} weight="bold" />
        Tools
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
        {tools.map((t, i) => (
          <div key={i}>
            <div style={{
              display: "flex", alignItems: "baseline", gap: 8,
              fontSize: "var(--text-xs)", fontFamily: "var(--font-mono)",
              lineHeight: 1.6,
            }}>
              <span style={{
                color: TOOL_COLORS[t.name] ?? "#888",
                fontWeight: 600, flexShrink: 0, minWidth: 40,
              }}>
                {t.name}
              </span>
              <span style={{ color: "#999", wordBreak: "break-all" }}>
                {t.summary}
              </span>
            </div>
            {t.old_text != null && t.new_text != null && (
              <DiffView oldText={t.old_text} newText={t.new_text} />
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
