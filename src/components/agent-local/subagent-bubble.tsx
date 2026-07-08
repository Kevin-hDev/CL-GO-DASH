import { useState } from "react";
import { useTranslation } from "react-i18next";
import { subagentColorKey, subagentDisplayName, subagentSecondaryText } from "@/lib/subagent-display";
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
                <span className={`sb-dot ${colorClass("sb-dot", agent)}`} />
                <span className="sb-agent-text">
                  <span className="sb-agent-name">{subagentDisplayName(agent)}</span>
                  <span className="sb-agent-description">
                    {subagentSecondaryText(agent)}
                  </span>
                </span>
              </button>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

function colorClass(prefix: string, agent: SubagentInfo): string {
  const key = subagentColorKey(agent);
  return `${prefix}-${key}`;
}
