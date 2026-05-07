import { useState } from "react";
import type { SubagentInfo } from "@/types/agent";
import "./subagent-accordion.css";

interface SubagentAccordionProps {
  subagents: SubagentInfo[];
  onCancel: (sessionId: string) => void;
  onOpen: (sessionId: string) => void;
}

export function SubagentAccordion({ subagents, onCancel, onOpen }: SubagentAccordionProps) {
  const [expanded, setExpanded] = useState(true);

  if (subagents.length === 0) return null;

  return (
    <div className="sa-accordion">
      <div className="sa-accordion-header" role="group">
        <button
          className="sa-accordion-toggle"
          onClick={() => setExpanded((v) => !v)}
          type="button"
        >
          <span className="sa-accordion-icon">⚙</span>
          <span className="sa-accordion-title">
            {subagents.length} agent{subagents.length > 1 ? "s" : ""} en arrière-plan
          </span>
          <span className={`sa-chevron ${expanded ? "sa-chevron-up" : ""}`}>›</span>
        </button>
        <button
          className="sa-stop-all"
          onClick={() => subagents.forEach((s) => onCancel(s.sessionId))}
          title="Tout arrêter"
          type="button"
        >
          ■
        </button>
      </div>
      <div className={`sa-accordion-body ${expanded ? "sa-expanded" : ""}`}>
        {subagents.map((agent) => (
          <div key={agent.sessionId} className="sa-agent-row">
            <span className={`sa-agent-dot sa-dot-${agent.type}`} />
            <span className="sa-agent-name">{agent.name}</span>
            <span className="sa-agent-status">
              {agent.status === "running" ? "en cours..." : agent.status}
            </span>
            <div className="sa-agent-actions">
              {agent.status === "running" && (
                <button
                  className="sa-btn-stop"
                  onClick={() => onCancel(agent.sessionId)}
                  title="Arrêter"
                  type="button"
                >
                  ■
                </button>
              )}
              <button
                className="sa-btn-open"
                onClick={() => onOpen(agent.sessionId)}
                type="button"
              >
                Ouvrir
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
