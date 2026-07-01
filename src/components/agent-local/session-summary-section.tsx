import type { ReactNode } from "react";
import { CaretDown } from "@/components/ui/icons";

interface SessionSummarySectionProps {
  title: string;
  count: number;
  open: boolean;
  onToggle: () => void;
  children: ReactNode;
}

export function SessionSummarySection({
  title,
  count,
  open,
  onToggle,
  children,
}: SessionSummarySectionProps) {
  return (
    <section className="ssb-section">
      <button className="ssb-section-toggle" type="button" aria-expanded={open} onClick={onToggle}>
        <span>{title} ({count})</span>
        <CaretDown
          className={`ssb-section-caret ${open ? "ssb-section-caret-open" : ""}`}
          size="var(--icon-sm)"
        />
      </button>
      <div className={`ssb-accordion ${open ? "ssb-accordion-open" : ""}`}>
        <div className="ssb-accordion-inner">{children}</div>
      </div>
    </section>
  );
}
