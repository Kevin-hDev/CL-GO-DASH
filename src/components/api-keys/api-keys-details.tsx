import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-shell";
import { Pencil, Trash, ArrowSquareOut } from "@/components/ui/icons";
import { SettingsCard } from "@/components/settings/settings-card";
import { ProviderIcon } from "@/lib/provider-icons";
import { getProviderDescription, type ProviderSpec } from "@/types/api";

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

  const handleDeleteClick = async () => {
    if (confirmDelete) {
      await onDelete();
      setConfirmDelete(false);
    } else {
      setConfirmDelete(true);
    }
  };

  return (
    <div style={{ padding: 24, overflowY: "auto", flex: 1 }}>
      <div style={{ maxWidth: 600, width: "100%", margin: "0 auto" }}>
        <div style={{
          display: "flex", alignItems: "flex-start",
          justifyContent: "space-between", marginBottom: 28,
        }}>
          <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
            <ProviderIcon providerId={provider.id} displayName={provider.display_name} size={36} />
            <div>
              <h2 style={{ fontSize: "var(--text-xl)", fontWeight: 700, color: "var(--ink)", margin: 0 }}>
                {provider.display_name}
              </h2>
              <div style={{ fontSize: "var(--text-xs)", color: "var(--ink-muted)", marginTop: 2 }}>
                {getProviderDescription(provider, i18n.language)}
              </div>
            </div>
          </div>
          <div style={{ display: "flex", gap: 4, alignItems: "center" }}>
            <button type="button" className="ak-connectors-btn" onClick={onAddConnector} style={{ marginRight: 8, whiteSpace: "nowrap" }}>
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
          <button type="button" className="ak-confirm-delete" onClick={handleDeleteClick} style={{ marginTop: 16 }}>
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
    <div style={{
      display: "flex", justifyContent: "space-between", alignItems: "center",
      padding: "10px 20px",
      borderBottom: last ? "none" : "1px solid var(--edge)",
      fontSize: "var(--text-sm)",
    }}>
      <span style={{
        color: "var(--ink-muted)", fontSize: "var(--text-xs)",
        fontWeight: 500, textTransform: "uppercase", letterSpacing: "0.5px",
      }}>
        {label}
      </span>
      {children ?? (
        <span style={{ color: "var(--ink)", fontFamily: "var(--font-mono)" }}>{value}</span>
      )}
    </div>
  );
}
