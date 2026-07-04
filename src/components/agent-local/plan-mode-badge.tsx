import { useTranslation } from "react-i18next";
import { X } from "@/components/ui/icons";
import "./plan-mode-badge.css";

interface PlanModeBadgeProps {
  onDisable: () => void;
}

export function PlanModeBadge({ onDisable }: PlanModeBadgeProps) {
  const { t } = useTranslation();
  return (
    <button
      type="button"
      className="pmb-root"
      onClick={onDisable}
      aria-label={t("chatMenu.disablePlanMode")}
      title={t("chatMenu.disablePlanMode")}
    >
      <span className="pmb-text">{t("chatMenu.planMode")}</span>
      <X size="var(--icon-2xs)" className="pmb-x" />
    </button>
  );
}
