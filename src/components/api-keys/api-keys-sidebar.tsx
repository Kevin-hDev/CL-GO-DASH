import { useTranslation } from "react-i18next";
import type { ProviderSpec } from "@/types/api";

interface ApiKeysSidebarProps {
  configured: ProviderSpec[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
}

export function ApiKeysSidebar({
  configured,
  selectedId,
  onSelect,
}: ApiKeysSidebarProps) {
  const { t } = useTranslation();

  return (
    <div className="ak-sidebar">
      <div className="ak-sidebar-header">{t("apiKeys.sidebar.configured")}</div>
      <div className="ak-sidebar-list">
        {configured.length === 0 ? (
          <div className="ak-sidebar-empty">{t("apiKeys.sidebar.empty")}</div>
        ) : (
          configured.map((p) => (
            <button
              key={p.id}
              type="button"
              className={`ak-sidebar-item ${selectedId === p.id ? "active" : ""}`}
              onClick={() => onSelect(p.id)}
            >
              <span className="ak-sidebar-dot" />
              <span>{p.display_name}</span>
            </button>
          ))
        )}
      </div>
    </div>
  );
}
