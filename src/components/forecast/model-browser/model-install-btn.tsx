import { useState, useCallback, useRef, useEffect } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";

interface DownloadProgress {
  model_name: string;
  downloaded: number;
  total: number;
  percent: number;
}

interface ModelInstallBtnProps {
  modelId: string;
  onDone: () => void;
}

export function ModelInstallBtn({ modelId, onDone }: ModelInstallBtnProps) {
  const [busy, setBusy] = useState(false);
  const [percent, setPercent] = useState(0);
  const cancelledRef = useRef(false);

  const handleInstall = useCallback(async () => {
    cancelledRef.current = false;
    setBusy(true);
    setPercent(0);

    const channel = new Channel<DownloadProgress>();
    channel.onmessage = (event: DownloadProgress) => {
      setPercent(Math.round(event.percent));
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
    return <span className="fmi-error">Erreur</span>;
  }

  return (
    <button className="fmi-btn" onClick={() => void handleInstall()}>
      Installer
    </button>
  );
}
