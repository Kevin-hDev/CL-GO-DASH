import "./chat.css";

interface TpsDisplayProps {
  tps: number;
  lastRequestTokens: number;
  isStreaming: boolean;
}

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

export function TpsDisplay({ tps, lastRequestTokens, isStreaming }: TpsDisplayProps) {
  if (lastRequestTokens < 1 && !isStreaming) return null;

  return (
    <div
      className="tps-display"
      style={{
        display: "flex", alignItems: "center", gap: 8,
        fontSize: "var(--text-xs)",
        color: "var(--ink-faint)",
        fontFamily: "var(--font-mono)",
        padding: "0 8px",
        whiteSpace: "nowrap",
      }}
      title="Tokens utilisés pendant la dernière requête"
    >
      {(isStreaming || tps > 0) && <span>{tps.toFixed(1)} t/s</span>}
      {lastRequestTokens > 0 && <span>{formatTokens(lastRequestTokens)} tokens</span>}
    </div>
  );
}
