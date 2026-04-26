import { useState, useCallback, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { invoke, Channel } from "@tauri-apps/api/core";
import { Check } from "@/components/ui/icons";
import { showToast } from "@/lib/toast-emitter";
import type { PullProgress } from "@/types/agent";
import "./ollama.css";

interface ModelInstallButtonProps {
  fullName: string;
  isInstalled: boolean;
  hasUpdate: boolean;
  sizeGb?: number;
}

const BTN_WIDTH = 88;

function isEscapeKey(e: KeyboardEvent): boolean {
  return e.code === "Escape";
}

export function ModelInstallButton({
  fullName, isInstalled, hasUpdate, sizeGb,
}: ModelInstallButtonProps) {
  const { t } = useTranslation();
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState("");
  const [percent, setPercent] = useState(0);
  const cancelledRef = useRef(false);

  const handleInstall = useCallback(async () => {
    if (sizeGb && sizeGb > 0) {
      const sizeBytes = Math.round(sizeGb * 1_073_741_824);
      const fits = await invoke<boolean>("check_model_fits_vram", { sizeBytes }).catch(() => true);
      if (!fits) {
        showToast(t("ollama.vramWarning"), "info", 4000);
      }
    }

    cancelledRef.current = false;
    setBusy(true);
    setStatus(t("ollama.starting"));
    setPercent(0);

    const channel = new Channel<PullProgress>();
    channel.onmessage = (event: PullProgress) => {
      setStatus(event.status);
      if (event.total && event.completed) {
        setPercent(Math.round((event.completed / event.total) * 100));
      }
    };

    try {
      await invoke("pull_ollama_model", {
        name: fullName,
        isUpdate: isInstalled && hasUpdate,
        onProgress: channel,
      });
      setStatus("");
    } catch {
      if (!cancelledRef.current) setStatus(t("ollama.pullError"));
    } finally {
      setBusy(false);
    }
  }, [fullName, isInstalled, hasUpdate, t]);

  const handleCancel = useCallback(async () => {
    cancelledRef.current = true;
    await invoke("cancel_pull_ollama_model").catch(() => {});
  }, []);

  useEffect(() => {
    if (!busy) return;
    const onEsc = (e: KeyboardEvent) => {
      if (isEscapeKey(e)) handleCancel();
    };
    window.addEventListener("keydown", onEsc);
    return () => window.removeEventListener("keydown", onEsc);
  }, [busy, handleCancel]);

  if (busy) {
    return (
      <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
        <div style={{ display: "flex", flexDirection: "column", alignItems: "flex-end", gap: 2 }}>
          <div className="ollama-progress-bar">
            <div className="ollama-progress-fill" style={{ width: `${percent}%` }} />
          </div>
          <span style={{ fontSize: "var(--text-xs)", color: "var(--ink-faint)" }}>
            {status}
          </span>
        </div>
        <button
          className="ollama-btn ollama-btn-cancel"
          style={{ width: BTN_WIDTH }}
          onClick={handleCancel}
        >
          {t("ollama.cancel")}
        </button>
      </div>
    );
  }

  if (isInstalled && !hasUpdate) {
    return (
      <div
        title={t("ollama.installedUpToDate")}
        style={{
          display: "flex", alignItems: "center",
          color: "var(--select-text)",
        }}
      >
        <Check size={18} />
      </div>
    );
  }

  if (isInstalled && hasUpdate) {
    return (
      <button
        className="ollama-btn ollama-btn-primary"
        style={{ width: BTN_WIDTH }}
        onClick={handleInstall}
        title={t("ollama.updateAvailable")}
      >
        {t("ollama.update")}
      </button>
    );
  }

  return (
    <button
      className="ollama-btn ollama-btn-primary"
      style={{ width: BTN_WIDTH }}
      onClick={handleInstall}
    >
      {t("ollama.install")}
    </button>
  );
}
