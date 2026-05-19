import { Archive } from "lucide-react";
import { useTranslation } from "react-i18next";

export function ContextCompressionMarker() {
  const { t } = useTranslation();
  return (
    <div className="msg-context-compressed" aria-label={t("agentLocal.contextCompressed")}>
      <Archive size={14} aria-hidden="true" />
      <span>{t("agentLocal.contextCompressed")}</span>
    </div>
  );
}
