import { useState, useCallback, useRef, useEffect } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { ConfirmButton } from "@/components/settings/confirm-button";

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
  const [removing, setRemoving] = useState(false);
  const [percent, setPercent] = useState(0);
  const cancelledRef = useRef(false);

  const handleInstall = useCallback(async () => {
    cancelledRef.current = false;
    setBusy(true);
    setPercent(0);

    const channel = new Channel<DownloadProgress>();
    channel.onmessage = (event: DownloadProgress) => {
      setPercent(Math.max(0, Math.min(100, Math.round(event.percent))));
    };

    try {
      await invoke("install_forecast_model", { name: modelId, onProgress: channel });
      onDone();
    } catch {
      if (!cancelledRef.current) setPercent(-1);
    } finally {
      setBusy(false);
    }
  }, [modelId, onDone]);

  const handleRemove = useCallback(async () => {
    setRemoving(true);
    try {
      await invoke("uninstall_forecast_model", { name: modelId });
      onDone();
    } finally {
      setRemoving(false);
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
          <div className="fmi-fill" style={{ width: `${percent}%` }} />
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
      <ConfirmButton
        className="fmi-btn fmi-btn-secondary"
        label={t("forecast.models.remove")}
        confirmLabel={t("forecast.models.confirmRemove")}
        onConfirm={() => void handleRemove()}
        disabled={removing}
      />
    );
  }

  return (
    <button className="fmi-btn" onClick={() => void handleInstall()}>
      {t("forecast.models.install")}
    </button>
  );
}
