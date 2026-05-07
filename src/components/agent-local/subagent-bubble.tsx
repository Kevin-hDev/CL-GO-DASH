import { useState } from "react";
import type { SubagentInfo } from "@/types/agent";
import "./subagent-bubble.css";

const BUBBLE_STYLE = {
  width: "100%", maxWidth: "720px",
  borderRadius: "var(--radius-md, 8px)",
  padding: "10px 14px", alignSelf: "center" as const, margin: "6px auto",
};

interface SubagentBubbleProps {
  subagents: SubagentInfo[];
  onOpen: (sessionId: string) => void;
}

export function SubagentBubble({ subagents, onOpen }: SubagentBubbleProps) {
  const [expanded, setExpanded] = useState(false);

  if (subagents.length === 0) return null;

  return (
    <div className="chat-bubble" style={BUBBLE_STYLE}>
      <button
        className="sb-header"
        onClick={() => setExpanded((v) => !v)}
        type="button"
      >
        <span className="sb-label">
          Sous-agent : {subagents.length} agent{subagents.length > 1 ? "s" : ""} créé{subagents.length > 1 ? "s" : ""}
        </span>
        <span className={`sb-chevron ${expanded ? "sb-chevron-up" : ""}`}>›</span>
      </button>
      {expanded && (
        <div className="sb-body">
          {subagents.map((agent) => (
            <button
              key={agent.sessionId}
              className="sb-agent-row"
              onClick={() => onOpen(agent.sessionId)}
              type="button"
            >
              <span className={`sb-dot sb-dot-${agent.type}`} />
              <span className="sb-agent-name">{agent.name}</span>
              <span className="sb-prompt-preview">
                {agent.promptPreview || "..."}
              </span>
            </button>
          ))}
        </div>
      )}
    </div>
  );
}
