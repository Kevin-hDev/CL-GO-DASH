import { useState, useEffect } from "react";

interface StreamingStatsProps {
  segmentStartedAt: number | null;
  liveTokenCount: number;
}

function formatElapsed(ms: number): string {
  const secs = Math.floor(ms / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  const rest = secs % 60;
  return `${mins}m ${rest}s`;
}

export function StreamingStats({ segmentStartedAt, liveTokenCount }: StreamingStatsProps) {
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
      ({formatElapsed(elapsed)}{hasTokens ? ` · ↑ ${liveTokenCount} tokens` : ""})
    </span>
  );
}

export function formatTotalElapsed(ms: number): string {
  if (ms <= 0) return "";
  const secs = Math.floor(ms / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  const rest = secs % 60;
  return `${mins}m ${rest}s`;
}
