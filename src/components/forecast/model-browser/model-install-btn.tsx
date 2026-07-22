import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { Check } from "@/components/ui/icons";
import { ConfirmButton } from "@/components/settings/confirm-button";
import { Tooltip } from "@/components/ui/tooltip";
import { useModelDownloads } from "@/hooks/use-model-downloads";
import { showToast } from "@/lib/toast-emitter";
import "../../ollama/ollama.css";
import "./model-install-btn.css";

interface ModelInstallBtnProps {
  modelId: string;
  installed: boolean;
  runtimeReady: boolean;
  allowUninstall?: boolean;
  onDone: () => void;
}

export function ModelInstallBtn({
  modelId,
  installed,
  runtimeReady,
  allowUninstall = false,
  onDone,
}: ModelInstallBtnProps) {
  const { t } = useTranslation();
  const [uninstalling, setUninstalling] = useState(false);
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

  const handleUninstall = useCallback(async () => {
    setUninstalling(true);
    try {
      await invoke("uninstall_forecast_model", { name: modelId });
      onDone();
    } catch {
      showToast(t("errors.operationFailed"), "error");
    } finally {
      setUninstalling(false);
    }
  }, [modelId, onDone, t]);

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
        <span className="fmi-phase">
          {t(`modelDownloads.phases.${ownDownload.phase}`)}
        </span>
        <div className="fmi-bar">
          <div className="fmi-fill" style={{ transform: `scaleX(${ownDownload.percent / 100})` }} />
        </div>
        <span className="fmi-pct">{ownDownload.percent}%</span>
        <button className="ollama-btn fmi-cancel" onClick={() => void handleCancel()}>
          {t("forecast.models.cancel")}
        </button>
      </div>
    );
  }

  if (installed && runtimeReady && allowUninstall) {
    return (
      <ConfirmButton
        className="ollama-btn fmi-btn fmi-uninstall"
        label={uninstalling ? t("forecast.models.uninstalling") : t("forecast.models.uninstall")}
        confirmLabel={t("forecast.models.confirmUninstall")}
        onConfirm={() => void handleUninstall()}
        disabled={uninstalling || blocked}
      />
    );
  }

  if (installed && runtimeReady) {
    return (
      <Tooltip label={t("forecast.models.installed")}>
        <div className="fmi-installed">
          <Check size="var(--icon-lg)" />
        </div>
      </Tooltip>
    );
  }

  return (
    <button
      className="ollama-btn fmi-btn"
      disabled={blocked}
      onClick={() => void handleInstall()}
    >
      {blocked
        ? t("modelDownloads.busy")
        : installed
          ? t("forecast.models.prepare")
          : t("forecast.models.install")}
    </button>
  );
}
