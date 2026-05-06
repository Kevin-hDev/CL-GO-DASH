import { useState } from "react";
import { useTranslation } from "react-i18next";
import "./messages.css";
import "./tool-bubble.css";

interface ThinkingSectionProps {
  content: string;
  durationMs?: number;
  isActive?: boolean;
}

export function ThinkingSection({ content, durationMs, isActive }: ThinkingSectionProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);

  if (!content) return null;

  const seconds = durationMs ? (durationMs / 1000).toFixed(1) : null;
  const label = seconds ? t("agentLocal.thinkingDuration", { seconds }) : t("agentLocal.thinking");

  return (
    <div>
      <button className={`thinking-toggle${isActive ? " thinking-active" : ""}`} onClick={() => setOpen(!open)}>
        <span className="tb-arrow">{open ? "▾" : "▸"}</span>
        <span>{label}</span>
      </button>
      <div className={`tb-accordion${open ? " tb-open" : ""}`}>
        <div className="tb-accordion-inner">
          <div className="thinking-content">{content}</div>
        </div>
      </div>
    </div>
  );
}
