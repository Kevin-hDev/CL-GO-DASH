import { useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Check } from "@/components/ui/icons";
import { useModelDownloads } from "@/hooks/use-model-downloads";

interface ModelInstallBtnProps {
  modelId: string;
  installed: boolean;
  onDone: () => void;
}

export function ModelInstallBtn({ modelId, installed, onDone }: ModelInstallBtnProps) {
  const { t } = useTranslation();
  const { activeDownload, startDownload, cancelDownload, downloads } = useModelDownloads();
  const ownDownload = activeDownload?.kind === "forecast" && activeDownload.modelId === modelId
    ? activeDownload
    : null;
  const blocked = Boolean(activeDownload && !ownDownload);
  const finishedOwn = downloads.find(
    (item) => item.kind === "forecast" && item.modelId === modelId && item.status === "completed",
  );

  useEffect(() => {
    if (finishedOwn) onDone();
  }, [finishedOwn, onDone]);

  const handleInstall = useCallback(async () => {
    await startDownload({ kind: "forecast", modelId }).catch(() => undefined);
  }, [modelId, startDownload]);

  const handleCancel = useCallback(async () => {
    if (ownDownload) await cancelDownload(ownDownload.id).catch(() => {});
  }, [cancelDownload, ownDownload]);

  useEffect(() => {
    if (!ownDownload) return;
    const onEsc = (e: KeyboardEvent) => {
      if (e.code === "Escape") void handleCancel();
    };
    window.addEventListener("keydown", onEsc);
    return () => window.removeEventListener("keydown", onEsc);
  }, [handleCancel, ownDownload]);

  if (ownDownload) {
    return (
      <div className="fmi-progress">
        <div className="fmi-bar">
          <div className="fmi-fill" style={{ transform: `scaleX(${ownDownload.percent / 100})` }} />
        </div>
        <span className="fmi-pct">{ownDownload.percent}%</span>
        <button className="fmi-cancel" onClick={() => void handleCancel()}>
          {t("forecast.models.cancel")}
        </button>
      </div>
    );
  }

  if (installed) {
    return (
      <div title={t("forecast.models.installed")} style={{ display: "flex", alignItems: "center", color: "var(--select-text)" }}>
        <Check size={18} />
      </div>
    );
  }

  return (
    <button className="fmi-btn" disabled={blocked} onClick={() => void handleInstall()}>
      {blocked ? t("modelDownloads.busy") : t("forecast.models.install")}
    </button>
  );
}
