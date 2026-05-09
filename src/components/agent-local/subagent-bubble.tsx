import { useState } from "react";
import { useTranslation } from "react-i18next";
import type { SubagentInfo } from "@/types/agent";
import "./subagent-bubble.css";

interface SubagentBubbleProps {
  subagents: SubagentInfo[];
  onOpen: (sessionId: string) => void;
}

export function SubagentBubble({ subagents, onOpen }: SubagentBubbleProps) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(false);

  if (subagents.length === 0) return null;

  return (
    <div className="chat-bubble">
      <button
        className="sb-header"
        onClick={() => setExpanded((v) => !v)}
        type="button"
      >
        <span className="sb-label">
          {t("subagents.bubbleLabel", { count: subagents.length })}
        </span>
        <span className={`sb-chevron ${expanded ? "sb-chevron-up" : ""}`}>›</span>
      </button>
      <div className={`tb-accordion${expanded ? " tb-open" : ""}`}>
        <div className="tb-accordion-inner">
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
        </div>
      </div>
    </div>
  );
}
