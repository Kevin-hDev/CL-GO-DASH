import { useTranslation } from "react-i18next";
import { Key } from "@/components/ui/icons";

export function ApiKeysEmpty() {
  const { t } = useTranslation();

  return (
    <div className="ak-empty">
      <Key size={48} className="ak-empty-icon" weight="thin" />
      <div className="ak-empty-title">{t("apiKeys.empty.title")}</div>
      <div className="ak-empty-subtitle">{t("apiKeys.empty.subtitle")}</div>
    </div>
  );
}
