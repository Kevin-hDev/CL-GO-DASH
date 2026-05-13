import type { ForecastDocPage } from "./forecast-docs-types";

interface ForecastDocsSidebarProps {
  pages: ForecastDocPage[];
  activeId: string;
  onSelect: (id: string) => void;
}

export function ForecastDocsSidebar({
  pages,
  activeId,
  onSelect,
}: ForecastDocsSidebarProps) {
  return (
    <aside className="fd-sidebar">
      <div className="fd-sidebar-kicker">Forecast</div>
      <nav className="fd-nav" aria-label="Documentation Forecast">
        {pages.map((page) => (
          <button
            key={page.id}
            className={`fd-nav-item ${page.id === activeId ? "fd-nav-item-active" : ""}`}
            type="button"
            onClick={() => onSelect(page.id)}
          >
            {page.navLabel}
          </button>
        ))}
      </nav>
    </aside>
  );
}
