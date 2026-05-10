import { useTranslation } from "react-i18next";
import { ProviderIcon } from "@/lib/provider-icons";
import type { ProviderSpec } from "@/types/api";

interface ApiKeysSidebarProps {
  configured: ProviderSpec[];
  selectedId: string | null;
  onSelect: (id: string | null) => void;
  onForecastModels?: () => void;
}

export function ApiKeysSidebar({
  configured,
  selectedId,
  onSelect,
  onForecastModels,
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
              <ProviderIcon providerId={p.id} displayName={p.display_name} size={18} />
              <span>{p.display_name}</span>
            </button>
          ))
        )}
      </div>
      {onForecastModels && (
        <>
          <div className="ak-sidebar-header" style={{ marginTop: 12 }}>
            {t("forecast.title")}
          </div>
          <div className="ak-sidebar-list">
            <button
              type="button"
              className="ak-sidebar-item"
              onClick={onForecastModels}
            >
              <span style={{ fontSize: 14 }}>📊</span>
              <span>{t("forecast.models.manage")}</span>
            </button>
          </div>
        </>
      )}
    </div>
  );
}
