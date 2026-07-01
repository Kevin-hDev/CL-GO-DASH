import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useLottie } from "lottie-react";
import thinkingAnimation from "@/assets/thinking-loader.json";
import { formatCompactDuration } from "@/lib/duration-format";

export function WorkingStats({ startedAt, liveTokenCount }: {
  startedAt: number; liveTokenCount: number;
}) {
  const { t } = useTranslation();
  const [now, setNow] = useState(startedAt);

  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 500);
    return () => clearInterval(id);
  }, []);

  const elapsed = formatCompactDuration(now - startedAt);
  const hasTokens = liveTokenCount > 0;

  return (
    <span className="working-stats thinking-active">
      <span>
        {t("agentLocal.working", { duration: elapsed })}
        {hasTokens ? ` · ↑ ${liveTokenCount} ${t("agentLocal.tokens")}` : ""}
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
