import { useState, useEffect, useCallback, useRef } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { PullProgress } from "@/types/agent";

const CHECK_INTERVAL_MS = 60 * 60 * 1000;

export interface AppUpdate {
  version: string;
  assetUrl: string;
}

interface DownloadProgress {
  completed: number;
  total: number;
}

export interface OllamaModelUpdate {
  fullName: string;
  family: string;
  tag: string;
}

export interface OllamaBinaryUpdate {
  currentVersion: string;
  latestVersion: string;
}

export interface PullingState {
  fullName: string;
  percent: number;
  status: string;
}

export function useUpdateChecker() {
  const [appUpdate, setAppUpdate] = useState<AppUpdate | null>(null);
  const [ollamaUpdates, setOllamaUpdates] = useState<OllamaModelUpdate[]>([]);
  const [pulling, setPulling] = useState<PullingState | null>(null);
  const pullingRef = useRef(false);
  const [ollamaBinaryUpdate, setOllamaBinaryUpdate] = useState<OllamaBinaryUpdate | null>(null);
  const [ollamaBinaryUpdating, setOllamaBinaryUpdating] = useState(false);
  const [ollamaBinaryPercent, setOllamaBinaryPercent] = useState(0);
  const [appDownloading, setAppDownloading] = useState(false);
  const [appPercent, setAppPercent] = useState(0);
  const timerRef = useRef<ReturnType<typeof setInterval>>(undefined);

  const checkAll = useCallback(async () => {
    try {
      const app = await invoke<AppUpdate | null>("check_app_update");
      setAppUpdate(app ?? null);
    } catch {
      /* network error, ignore */
    }

    try {
      const models = await invoke<OllamaModelUpdate[]>("check_ollama_updates");
      setOllamaUpdates(models);
    } catch {
      /* ollama not running, ignore */
    }

    try {
      const binary = await invoke<OllamaBinaryUpdate | null>("check_ollama_binary_update");
      setOllamaBinaryUpdate(binary ?? null);
    } catch {
      /* ollama not running or network error, ignore */
    }
  }, []);

  useEffect(() => {
    // eslint-disable-next-line react-hooks/set-state-in-effect -- checkAll is async, setState in callbacks only
    void checkAll();
    timerRef.current = setInterval(() => void checkAll(), CHECK_INTERVAL_MS);
    const unlisten = listen("ollama-models-changed", () => {
      if (pullingRef.current) return;
      invoke<OllamaModelUpdate[]>("check_ollama_updates")
        .then(setOllamaUpdates)
        .catch(() => {});
    });
    return () => {
      clearInterval(timerRef.current);
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [checkAll]);

  const downloadAppUpdate = useCallback(async (assetUrl: string) => {
    setAppDownloading(true);
    setAppPercent(0);

    const channel = new Channel<DownloadProgress>();
    channel.onmessage = (event: DownloadProgress) => {
      const pct = event.total > 0
        ? Math.round((event.completed / event.total) * 100)
        : 0;
      setAppPercent(pct);
    };

    try {
      await invoke("download_app_update", { assetUrl, onProgress: channel });
      setAppUpdate(null);
    } catch {
      /* download failed */
    } finally {
      setAppDownloading(false);
    }
  }, []);

  const pullModel = useCallback(async (fullName: string) => {
    pullingRef.current = true;
    setPulling({ fullName, percent: 0, status: "" });

    const channel = new Channel<PullProgress>();
    channel.onmessage = (event: PullProgress) => {
      const pct = event.total && event.completed
        ? Math.round((event.completed / event.total) * 100)
        : 0;
      setPulling({ fullName, percent: pct, status: event.status });
    };

    try {
      await invoke("pull_ollama_model", { name: fullName, isUpdate: true, onProgress: channel });
      setOllamaUpdates((prev) => prev.filter((u) => u.fullName !== fullName));
    } catch {
      /* pull failed */
    } finally {
      pullingRef.current = false;
      setPulling(null);
    }
  }, []);

  const updateOllamaBinary = useCallback(async () => {
    if (!ollamaBinaryUpdate) return;
    setOllamaBinaryUpdating(true);
    setOllamaBinaryPercent(0);

    interface SetupProgress { completed: number; total: number; status: string }
    const channel = new Channel<SetupProgress>();
    channel.onmessage = (event: SetupProgress) => {
      if (event.status === "restarting") {
        setOllamaBinaryPercent(100);
        return;
      }
      const pct = event.total > 0
        ? Math.round((event.completed / event.total) * 100)
        : 0;
      setOllamaBinaryPercent(pct);
    };

    try {
      await invoke("update_ollama_binary", {
        version: ollamaBinaryUpdate.latestVersion,
        onProgress: channel,
      });
      setOllamaBinaryUpdate(null);
    } catch {
      /* update failed */
    } finally {
      setOllamaBinaryUpdating(false);
    }
  }, [ollamaBinaryUpdate]);

  const totalCount =
    (appUpdate ? 1 : 0) +
    (ollamaBinaryUpdate ? 1 : 0) +
    ollamaUpdates.length;

  return {
    appUpdate, ollamaUpdates, pulling,
    ollamaBinaryUpdate, ollamaBinaryUpdating, ollamaBinaryPercent,
    appDownloading, appPercent,
    totalCount, pullModel, downloadAppUpdate, updateOllamaBinary, checkAll,
  };
}
