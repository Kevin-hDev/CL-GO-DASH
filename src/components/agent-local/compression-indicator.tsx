import { useLottie } from "lottie-react";
import { useTranslation } from "react-i18next";
import thinkingAnimation from "@/assets/thinking-loader.json";
import "./compression-indicator.css";

interface CompressionIndicatorProps {
  label?: string;
}

export function CompressionIndicator({ label }: CompressionIndicatorProps) {
  const { t } = useTranslation();
  const { View } = useLottie({
    animationData: thinkingAnimation,
    loop: true,
    className: "chat-loading-lottie",
  });

  return (
    <div className="compression-indicator">
      {View}
      <span className="compression-label">{label ?? t("agentLocal.compression")}</span>
    </div>
  );
}
