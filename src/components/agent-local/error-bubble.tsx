import { useState, useEffect, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { useTranslation } from "react-i18next";
import { showToast } from "@/lib/toast-emitter";

const BUBBLE_STYLE = {
  width: "100%", maxWidth: "var(--bubble-max-width, 660px)",
  borderRadius: "var(--radius-md, 8px)",
  padding: "var(--space-sm, 10px) var(--space-md, 14px)",
  alignSelf: "center" as const, margin: "var(--space-xs, 6px) auto",
  fontSize: "var(--font-size-sm, 12px)",
  fontFamily: "var(--font-mono, monospace)", lineHeight: 1.5,
  wordBreak: "break-word" as const,
};

interface ErrorBubbleProps {
  message: string;
  isConnection?: boolean;
  onRetry?: () => void;
}

export function ErrorBubble({ message, isConnection, onRetry }: ErrorBubbleProps) {
  const { t } = useTranslation();

  if (isConnection && onRetry) {
    return <ConnectionErrorBubble onRetry={onRetry} />;
  }

  return (
    <div style={{
      ...BUBBLE_STYLE,
      background: "var(--signal-error-bg)",
      border: "1px solid color-mix(in srgb, var(--signal-error) 20%, transparent)",
      color: "var(--signal-error)",
    }}>
      {message === "ollama_connection_lost" ? t("errors.ollamaConnectionLost") : message}
    </div>
  );
}

function ConnectionErrorBubble({ onRetry }: { onRetry: () => void }) {
  const { t } = useTranslation();
  const [elapsed, setElapsed] = useState(0);
  const [ollamaUp, setOllamaUp] = useState(false);
  const [countdown, setCountdown] = useState(0);
  const retried = useRef(false);

  useEffect(() => {
    if (ollamaUp) return;
    const interval = setInterval(() => setElapsed((p) => p + 1), 1000);
    return () => clearInterval(interval);
  }, [ollamaUp]);

  useEffect(() => {
    const unlisten = listen<boolean>("ollama-status", (e) => {
      if (e.payload && !retried.current) {
        setOllamaUp(true);
        showToast(t("errors.ollamaReconnected"), "success");
      }
    });
    return () => { unlisten.then((fn) => fn()).catch(() => {}); };
  }, [t]);

  useEffect(() => {
    if (!ollamaUp || retried.current) return;
    setCountdown(3);
    const interval = setInterval(() => {
      setCountdown((prev) => {
        if (prev <= 1) {
          clearInterval(interval);
          retried.current = true;
          onRetry();
          return 0;
        }
        return prev - 1;
      });
    }, 1000);
    return () => clearInterval(interval);
  }, [ollamaUp, onRetry]);

  if (ollamaUp) {
    return (
      <div style={{
        ...BUBBLE_STYLE,
        background: "var(--signal-ok-bg)",
        border: "1px solid color-mix(in srgb, var(--signal-ok) 20%, transparent)",
        color: "var(--signal-ok)",
      }}>
        {t("errors.ollamaReconnecting", { seconds: countdown })}
      </div>
    );
  }

  return (
    <div style={{
      ...BUBBLE_STYLE,
      background: "var(--signal-error-bg)",
      border: "1px solid color-mix(in srgb, var(--signal-error) 20%, transparent)",
      color: "var(--signal-error)",
    }}>
      {t("errors.ollamaWaiting", { seconds: elapsed })}
    </div>
  );
}
