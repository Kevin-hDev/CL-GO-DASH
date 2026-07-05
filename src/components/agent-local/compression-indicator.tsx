import { useTranslation } from "react-i18next";
import { InlineActivityIndicator } from "./inline-activity-indicator";
import "./compression-indicator.css";

interface CompressionIndicatorProps {
  label?: string;
}

export function CompressionIndicator({ label }: CompressionIndicatorProps) {
  const { t } = useTranslation();

  return (
    <div className="compression-indicator">
      <InlineActivityIndicator className="compression-label">
        {label ?? t("agentLocal.compression")}
      </InlineActivityIndicator>
    </div>
  );
}
