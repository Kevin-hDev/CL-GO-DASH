import "./conversation.css";

interface OllamaIndicatorProps {
  running: boolean;
}

export function OllamaIndicator({ running }: OllamaIndicatorProps) {
  return (
    <span className={`ollama-dot ${running ? "connected" : "disconnected"}`} />
  );
}
