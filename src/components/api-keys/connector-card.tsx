import { Plus, Check, Key } from "@/components/ui/icons";
import { ProviderIcon } from "@/lib/provider-icons";
import type { ProviderSpec } from "@/types/api";

interface ConnectorCardProps {
  provider: ProviderSpec;
  configured: boolean;
  onAdd: () => void;
}

export function ConnectorCard({
  provider,
  configured,
  onAdd,
}: ConnectorCardProps) {
  return (
    <button
      type="button"
      className={`ak-connector-card ${configured ? "configured" : ""}`}
      onClick={configured ? undefined : onAdd}
      disabled={configured}
    >
      <ProviderIcon
        providerId={provider.id}
        displayName={provider.display_name}
        size={40}
      />
      <div className="ak-connector-card-body">
        <div className="ak-connector-card-name">{provider.display_name}</div>
        <div className="ak-connector-card-desc">{provider.short_description}</div>
        <div className="ak-connector-card-meta">
          <span className="ak-connector-card-cat">
            {provider.category.toUpperCase()}
          </span>
          <span className="ak-connector-card-tier">
            {provider.free_tier_label}
          </span>
          <Key size={12} className="ak-connector-card-keyicon" weight="fill" />
        </div>
      </div>
      <div className={`ak-connector-card-action ${configured ? "done" : ""}`}>
        {configured ? <Check size={16} weight="bold" /> : <Plus size={16} weight="bold" />}
      </div>
    </button>
  );
}
