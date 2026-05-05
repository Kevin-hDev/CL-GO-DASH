import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretRight, CaretDown } from "@/components/ui/icons";
import "./messages.css";

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
        {open ? <CaretDown size={12} /> : <CaretRight size={12} />}
        <span>{label}</span>
      </button>
      {open && <div className="thinking-content">{content}</div>}
    </div>
  );
}
