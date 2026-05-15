import { useState, useCallback, useRef, useEffect } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Check } from "@/components/ui/icons";

interface DownloadProgress {
  model_name: string;
  downloaded: number;
  total: number;
  percent: number;
}

interface ModelInstallBtnProps {
  modelId: string;
  installed: boolean;
  onDone: () => void;
}

export function ModelInstallBtn({ modelId, installed, onDone }: ModelInstallBtnProps) {
  const { t } = useTranslation();
  const [busy, setBusy] = useState(false);
  const [percent, setPercent] = useState(0);
  const cancelledRef = useRef(false);

  const handleInstall = useCallback(async () => {
    cancelledRef.current = false;
    setBusy(true);
    setPercent(0);

    const channel = new Channel<DownloadProgress>();
    channel.onmessage = (event: DownloadProgress) => {
      const next = Math.max(0, Math.min(100, Math.round(event.percent)));
      setPercent((current) => Math.max(current, next));
    };

    try {
      await invoke("install_forecast_model", { name: modelId, onProgress: channel });
      setPercent(100);
      onDone();
    } catch {
      if (!cancelledRef.current) setPercent(-1);
    } finally {
      setBusy(false);
    }
  }, [modelId, onDone]);

  useEffect(() => {
    if (!busy) return;
    const onEsc = (e: KeyboardEvent) => {
      if (e.code === "Escape") cancelledRef.current = true;
    };
    window.addEventListener("keydown", onEsc);
    return () => window.removeEventListener("keydown", onEsc);
  }, [busy]);

  if (busy) {
    return (
      <div className="fmi-progress">
        <div className="fmi-bar">
          <div className="fmi-fill" style={{ transform: `scaleX(${percent / 100})` }} />
        </div>
        <span className="fmi-pct">{percent}%</span>
      </div>
    );
  }

  if (percent === -1) {
    return <span className="fmi-error">{t("forecast.models.installError")}</span>;
  }

  if (installed) {
    return (
      <div
        title={t("forecast.models.installed")}
        style={{ display: "flex", alignItems: "center", color: "var(--select-text)" }}
      >
        <Check size={18} />
      </div>
    );
  }

  return (
    <button className="fmi-btn" onClick={() => void handleInstall()}>
      {t("forecast.models.install")}
    </button>
  );
}
