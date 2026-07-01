import { useState, useEffect } from "react";
import { formatCompactDuration } from "@/lib/duration-format";

interface StreamingStatsProps {
  segmentStartedAt: number | null;
  liveTokenCount: number;
}

export function StreamingStats({ segmentStartedAt, liveTokenCount }: StreamingStatsProps) {
  // eslint-disable-next-line react-hooks/purity -- Date.now() as initial state is standard React pattern
  const [now, setNow] = useState(Date.now());

  useEffect(() => {
    if (!segmentStartedAt) return;
    const id = setInterval(() => setNow(Date.now()), 500);
    return () => clearInterval(id);
  }, [segmentStartedAt]);

  if (!segmentStartedAt) return null;

  const elapsed = now - segmentStartedAt;
  const hasTokens = liveTokenCount > 0;

  return (
    <span style={{
      color: "var(--ink-faint)",
      fontSize: "11px",
      fontFamily: "var(--font-mono, monospace)",
      marginLeft: 8,
      opacity: 0.7,
    }}>
      ({formatCompactDuration(elapsed)}{hasTokens ? ` · ↑ ${liveTokenCount} tokens` : ""})
    </span>
  );
}

export function formatTotalElapsed(ms: number): string {
  if (ms <= 0) return "";
  return formatCompactDuration(ms);
}
