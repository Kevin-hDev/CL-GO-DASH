import { useTranslation } from "react-i18next";
import type { ForecastSection } from "@/hooks/use-forecast-panel";

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
  onSelect: (section: ForecastSection) => void;
}

export function ForecastNav({ open, activeSection, onSelect }: ForecastNavProps) {
  const { t } = useTranslation();

  return (
    <div
      className="fc-nav"
      style={{
        maxHeight: open ? 200 : 0,
        opacity: open ? 1 : 0,
        overflow: "hidden",
        transition: open
          ? "max-height 250ms ease-out, opacity 200ms ease-out"
          : "max-height 200ms ease-in, opacity 150ms ease-in",
      }}
    >
      <div className="fc-nav-list">
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
