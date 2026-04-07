import { useState } from "react";
import { CaretRight, CaretDown } from "@/components/ui/icons";
import "./messages.css";

interface ThinkingSectionProps {
  content: string;
  durationMs?: number;
}

export function ThinkingSection({ content, durationMs }: ThinkingSectionProps) {
  const [open, setOpen] = useState(false);

  if (!content) return null;

  const seconds = durationMs ? (durationMs / 1000).toFixed(1) : null;
  const label = seconds ? `A réfléchi pendant ${seconds}s` : "Réflexion";

  return (
    <div>
      <button className="thinking-toggle" onClick={() => setOpen(!open)}>
        {open ? <CaretDown size={12} /> : <CaretRight size={12} />}
        <span>{label}</span>
      </button>
      {open && <div className="thinking-content">{content}</div>}
    </div>
  );
}
