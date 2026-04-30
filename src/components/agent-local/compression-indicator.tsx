import { useLottie } from "lottie-react";
import thinkingAnimation from "@/assets/thinking-loader.json";
import "./compression-indicator.css";

export function CompressionIndicator() {
  const { View } = useLottie({
    animationData: thinkingAnimation,
    loop: true,
    className: "chat-loading-lottie",
  });

  return (
    <div className="compression-indicator">
      {View}
      <span className="compression-label">Compression</span>
    </div>
  );
}
