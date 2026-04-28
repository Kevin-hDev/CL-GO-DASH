import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { Pencil, Trash, ArrowSquareOut } from "@/components/ui/icons";
import { SettingsCard } from "@/components/settings/settings-card";
import { ProviderIcon } from "@/lib/provider-icons";
import { getProviderDescription, type ProviderSpec } from "@/types/api";
import "./api-keys-details.css";

interface ProviderQuota {
  available: boolean;
  label: string;
}

interface ApiKeysDetailsProps {
  provider: ProviderSpec;
  onEdit: () => void;
  onDelete: () => Promise<void>;
  onAddConnector: () => void;
}

export function ApiKeysDetails({ provider, onEdit, onDelete, onAddConnector }: ApiKeysDetailsProps) {
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
      .catch((e) => console.warn("Quota load:", e))
      .finally(() => { if (!cancelled) setQuotaLoading(false); });
    return () => { cancelled = true; };
  }, [provider.id]);

  useEffect(() => {
    if (!confirmDelete) return;
    const timer = setTimeout(() => setConfirmDelete(false), 5000);
    const onKey = (e: KeyboardEvent) => {
      if (e.key.startsWith("Esc")) {
        e.preventDefault();
        setConfirmDelete(false);
      }
    };
    window.addEventListener("keydown", onKey);
    return () => {
      clearTimeout(timer);
      window.removeEventListener("keydown", onKey);
    };
  }, [confirmDelete]);

  const handleDeleteClick = async () => {
    if (confirmDelete) {
      await onDelete();
      setConfirmDelete(false);
    } else {
      setConfirmDelete(true);
    }
  };

  return (
    <div className="akd-scroll">
      <div className="akd-inner">
        <div className="akd-header">
          <div className="akd-provider-info">
            <ProviderIcon providerId={provider.id} displayName={provider.display_name} size={36} />
            <div>
              <h2 className="akd-provider-name">
                {provider.display_name}
              </h2>
              <div className="akd-provider-desc">
                {getProviderDescription(provider, i18n.language)}
              </div>
            </div>
          </div>
          <div className="akd-header-actions">
            <button type="button" className="ak-connectors-btn" onClick={onAddConnector}>
              {t("apiKeys.main.connectorsBtn")}
            </button>
            <button type="button" className="ak-icon-btn" onClick={onEdit} title={t("apiKeys.details.edit")}>
              <Pencil size={16} />
            </button>
            <button type="button" className="ak-icon-btn danger" onClick={() => setConfirmDelete(true)} title={t("apiKeys.details.delete")}>
              <Trash size={16} />
            </button>
          </div>
        </div>

        <SettingsCard>
          <DetailRow label={t("apiKeys.details.freeTier")} value={provider.free_tier_label} />
          <DetailRow
            label={t("apiKeys.details.quota")}
            value={quotaLoading ? "..." : quota ? quota.label : t("apiKeys.details.quotaUnavailable")}
          />
          <DetailRow label={t("apiKeys.details.signupLink")}>
            <button type="button" className="ak-signup-link" onClick={() => open(provider.signup_url)}>
              {t("apiKeys.details.openSite")} <ArrowSquareOut size={12} />
            </button>
          </DetailRow>
          <DetailRow label={t("apiKeys.details.apiKey")} value="••••••••" last />
        </SettingsCard>

        {confirmDelete && (
          <button type="button" className="ak-confirm-delete" onClick={handleDeleteClick}>
            {t("apiKeys.details.confirmDelete")}
          </button>
        )}
      </div>
    </div>
  );
}

function DetailRow({ label, value, children, last }: {
  label: string; value?: string; children?: React.ReactNode; last?: boolean;
}) {
  return (
    <div className={`akd-row ${last ? "" : "akd-row-border"}`}>
      <span className="akd-row-label">
        {label}
      </span>
      {children ?? (
        <span className="akd-row-value">{value}</span>
      )}
    </div>
  );
}
