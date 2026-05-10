import type { ForecastSection } from "@/hooks/use-forecast-panel";

const NAV_ITEMS: { id: ForecastSection; label: string }[] = [
  { id: "view", label: "Vue principale" },
  { id: "scenarios", label: "Scénarios" },
  { id: "analysis", label: "Analyse" },
  { id: "notes", label: "Notes" },
  { id: "history", label: "Historique" },
];

interface ForecastNavProps {
  open: boolean;
  activeSection: ForecastSection;
  onSelect: (section: ForecastSection) => void;
}

export function ForecastNav({ open, activeSection, onSelect }: ForecastNavProps) {
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
            {item.label}
          </button>
        ))}
      </div>
    </div>
  );
}
