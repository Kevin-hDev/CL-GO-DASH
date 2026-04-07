import "./chat.css";

interface TpsDisplayProps {
  tps: number;
  tokenCount: number;
  isStreaming: boolean;
}

export function TpsDisplay({ tps, tokenCount, isStreaming }: TpsDisplayProps) {
  if (tokenCount < 1 && !isStreaming) return null;

  const formattedTokens = tokenCount >= 1000
    ? `${(tokenCount / 1000).toFixed(1)}K`
    : String(tokenCount);

  return (
    <div className="tps-display">
      {(isStreaming || tps > 0) && <span>{tps.toFixed(1)} t/s</span>}
      <span>{formattedTokens} tokens</span>
    </div>
  );
}
