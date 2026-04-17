import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { Key, Pencil, Trash, ArrowSquareOut } from "@/components/ui/icons";
import { getProviderDescription, type ProviderSpec } from "@/types/api";

interface ProviderQuota {
  available: boolean;
  label: string;
}

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
  const { t, i18n } = useTranslation();
  const [confirmDelete, setConfirmDelete] = useState(false);
  const [quota, setQuota] = useState<ProviderQuota | null>(null);
  const [quotaLoading, setQuotaLoading] = useState(false);

  useEffect(() => {
    let cancelled = false;
    setQuota(null);
    setQuotaLoading(true);
    invoke<ProviderQuota | null>("get_provider_quota", { providerId: provider.id })
      .then((q) => { if (!cancelled) setQuota(q); })
      .catch(() => {})
      .finally(() => { if (!cancelled) setQuotaLoading(false); });
    return () => { cancelled = true; };
  }, [provider.id]);

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
              {getProviderDescription(provider, i18n.language)}
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
          <span className="ak-detail-label">{t("apiKeys.details.quota")}</span>
          <span className="ak-detail-value">
            {quotaLoading
              ? "..."
              : quota
                ? quota.label
                : t("apiKeys.details.quotaUnavailable")}
          </span>
        </div>

        <div className="ak-detail-row">
          <span className="ak-detail-label">{t("apiKeys.details.signupLink")}</span>
          <button
            type="button"
            className="ak-signup-link"
            onClick={() => open(provider.signup_url)}
          >
            {t("apiKeys.details.openSite")} <ArrowSquareOut size={12} />
          </button>
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
