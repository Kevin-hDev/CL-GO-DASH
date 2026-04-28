import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { invoke, Channel } from "@tauri-apps/api/core";
import "./ollama.css";
import "./ollama-setup-screen.css";

interface OllamaSetupProgress {
  completed: number;
  total: number;
  status: string;
}

interface OllamaSetupScreenProps {
  onComplete: () => void;
}

export function OllamaSetupScreen({ onComplete }: OllamaSetupScreenProps) {
  const { t } = useTranslation();
  const [downloading, setDownloading] = useState(false);
  const [percent, setPercent] = useState(0);
  const [status, setStatus] = useState("");
  const [error, setError] = useState<string | null>(null);

  const handleDownload = useCallback(async () => {
    setDownloading(true);
    setError(null);
    setPercent(0);

    const channel = new Channel<OllamaSetupProgress>();
    channel.onmessage = (event) => {
      setStatus(event.status);
      if (event.total > 0) {
        setPercent(Math.round((event.completed / event.total) * 100));
      }
    };

    try {
      await invoke("download_ollama", { onProgress: channel });
      onComplete();
    } catch (e) {
      setError(String(e));
      setDownloading(false);
    }
  }, [onComplete]);

  return (
    <div className="oss-container">
      <h2 className="oss-title">
        {t("ollamaSetup.title")}
      </h2>
      <p className="oss-description">
        {t("ollamaSetup.description")}
      </p>

      {downloading ? (
        <div className="oss-download-block">
          <div className="ollama-progress-bar oss-progress-bar">
            <div className="ollama-progress-fill" style={{ width: `${percent}%` }} />
          </div>
          <span className="oss-status-text">
            {status === "extracting"
              ? t("ollamaSetup.extracting")
              : status === "downloading-rocm"
                ? `${t("ollamaSetup.downloadingGpu")} ${percent}%`
                : `${percent}%`}
          </span>
        </div>
      ) : (
        <button
          className="ollama-btn ollama-btn-primary oss-download-btn"
          onClick={handleDownload}
        >
          {t("ollamaSetup.download")}
        </button>
      )}

      {error && (
        <div className="oss-error-block">
          <span className="oss-error-label">
            {t("ollamaSetup.error")}
          </span>
          <p className="oss-error-detail">
            {error}
          </p>
        </div>
      )}
    </div>
  );
}
