import { useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { ChevronDown } from "lucide-react";
import type { ForecastSection } from "@/hooks/use-forecast-panel";
import "./forecast-nav.css";

const NAV_ITEMS: { id: ForecastSection; i18nKey: string }[] = [
  { id: "view", i18nKey: "forecast.nav.mainView" },
  { id: "scenarios", i18nKey: "forecast.nav.scenarios" },
  { id: "analysis", i18nKey: "forecast.nav.analysis" },
  { id: "notes", i18nKey: "forecast.nav.notes" },
  { id: "history", i18nKey: "forecast.nav.history" },
];

interface ForecastNavProps {
  open: boolean;
  activeSection: ForecastSection;
  onToggle: () => void;
  onSelect: (section: ForecastSection) => void;
}

export function ForecastNav({ open, activeSection, onToggle, onSelect }: ForecastNavProps) {
  const { t } = useTranslation();
  const rootRef = useRef<HTMLDivElement | null>(null);
  const activeItem = NAV_ITEMS.find((item) => item.id === activeSection) ?? NAV_ITEMS[0];

  useEffect(() => {
    if (!open) return;

    const handlePointerDown = (event: MouseEvent) => {
      if (!rootRef.current?.contains(event.target as Node)) onToggle();
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onToggle();
    };

    window.addEventListener("mousedown", handlePointerDown);
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("mousedown", handlePointerDown);
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [open, onToggle]);

  return (
    <div ref={rootRef} className="fc-nav-root">
      <button className={`fc-nav-trigger ${open ? "is-open" : ""}`} onClick={onToggle}>
        <span className="fc-nav-label">{t(activeItem.i18nKey)}</span>
        <ChevronDown size={14} className={`fc-nav-chevron ${open ? "is-open" : ""}`} />
      </button>
      <div className={`fc-nav-panel ${open ? "is-open" : ""}`}>
        {NAV_ITEMS.map((item) => (
          <button
            key={item.id}
            className={`fc-nav-item ${activeSection === item.id ? "fc-nav-active" : ""}`}
            onClick={() => onSelect(item.id)}
          >
            <span
              className="fc-nav-dot"
              style={{
                background: activeSection === item.id ? "var(--fc-nav-active)" : "transparent",
              }}
            />
            {t(item.i18nKey)}
          </button>
        ))}
      </div>
    </div>
  );
}
