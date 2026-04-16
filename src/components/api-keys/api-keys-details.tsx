import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Key, Pencil, Trash, ArrowSquareOut } from "@/components/ui/icons";
import type { ProviderSpec } from "@/types/api";

interface ApiKeysDetailsProps {
  provider: ProviderSpec;
  onEdit: () => void;
  onDelete: () => Promise<void>;
}

export function ApiKeysDetails({
  provider,
  onEdit,
  onDelete,
}: ApiKeysDetailsProps) {
  const { t } = useTranslation();
  const [confirmDelete, setConfirmDelete] = useState(false);

  const handleDeleteClick = async () => {
    if (confirmDelete) {
      await onDelete();
      setConfirmDelete(false);
    } else {
      setConfirmDelete(true);
    }
  };

  return (
    <div className="ak-detail">
      <div className="ak-detail-header">
        <div className="ak-detail-info">
          <div className="ak-detail-icon">
            <Key size={28} weight="regular" />
          </div>
          <div className="ak-detail-text">
            <div className="ak-detail-name">{provider.display_name}</div>
            <div className="ak-detail-description">
              {provider.short_description}
            </div>
            <div className="ak-detail-badges">
              <span className="ak-badge">{provider.category.toUpperCase()}</span>
              <span className="ak-badge ak-badge-success">
                {t("apiKeys.details.connected")}
              </span>
            </div>
          </div>
        </div>
        <div className="ak-detail-actions">
          <button
            type="button"
            className="ak-icon-btn"
            onClick={onEdit}
            title={t("apiKeys.details.edit")}
          >
            <Pencil size={16} />
          </button>
          <button
            type="button"
            className="ak-icon-btn danger"
            onClick={() => setConfirmDelete(true)}
            title={t("apiKeys.details.delete")}
          >
            <Trash size={16} />
          </button>
        </div>
      </div>

      <div className="ak-detail-body">
        <div className="ak-detail-row">
          <span className="ak-detail-label">{t("apiKeys.details.freeTier")}</span>
          <span className="ak-detail-value">{provider.free_tier_label}</span>
        </div>
        <div className="ak-detail-row">
          <span className="ak-detail-label">{t("apiKeys.details.signupLink")}</span>
          <a
            href={provider.signup_url}
            target="_blank"
            rel="noreferrer"
            className="ak-signup-link"
          >
            {t("apiKeys.details.openSite")} <ArrowSquareOut size={12} />
          </a>
        </div>
        <div className="ak-detail-row">
          <span className="ak-detail-label">{t("apiKeys.details.apiKey")}</span>
          <span className="ak-detail-value">••••••••</span>
        </div>

        {confirmDelete && (
          <button
            type="button"
            className="ak-confirm-delete"
            onClick={handleDeleteClick}
          >
            {t("apiKeys.details.confirmDelete")}
          </button>
        )}
      </div>
    </div>
  );
}
