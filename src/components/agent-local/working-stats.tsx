import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useLottie } from "lottie-react";
import thinkingAnimation from "@/assets/thinking-loader.json";

export function WorkingStats({ startedAt, liveTokenCount }: {
  startedAt: number; liveTokenCount: number;
}) {
  const { t } = useTranslation();
  const [now, setNow] = useState(Date.now());

  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 500);
    return () => clearInterval(id);
  }, []);

  const elapsed = Math.max(0, Math.floor((now - startedAt) / 1000));
  const hasTokens = liveTokenCount > 0;

  return (
    <span className="working-stats thinking-active">
      <span>
        {t("agentLocal.working", { seconds: elapsed })}
        {hasTokens ? ` · ↑ ${liveTokenCount} tokens` : ""}
      </span>
    </span>
  );
}

export function LoadingIndicator({ startedAt, liveTokenCount }: {
  startedAt: number; liveTokenCount: number;
}) {
  const { View } = useLottie({
    animationData: thinkingAnimation, loop: true, className: "chat-loading-lottie",
  });
  return (
    <div className="chat-loading">
      {View}
      <WorkingStats startedAt={startedAt} liveTokenCount={liveTokenCount} />
    </div>
  );
}
