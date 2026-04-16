import "./chat.css";

interface TpsDisplayProps {
  tps: number;
  isStreaming: boolean;
}

/**
 * Indicateur de vitesse en temps réel pendant le streaming.
 * Les tokens consommés par la dernière requête sont affichés sous chaque
 * réponse assistant (voir MessageList/AssistantMessage).
 */
export function TpsDisplay({ tps, isStreaming }: TpsDisplayProps) {
  if (!isStreaming && tps < 0.1) return null;

  return (
    <div
      className="tps-display"
      style={{
        display: "flex",
        alignItems: "center",
        gap: 8,
        fontSize: "var(--text-xs)",
        color: "var(--ink-faint)",
        fontFamily: "var(--font-mono)",
        padding: "0 8px",
        whiteSpace: "nowrap",
      }}
      title="Vitesse du streaming"
    >
      {(isStreaming || tps > 0) && <span>{tps.toFixed(1)} t/s</span>}
    </div>
  );
}
