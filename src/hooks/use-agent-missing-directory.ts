import { useCallback, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";

export interface MissingSessionDirectory {
  missing_path: string;
  nearest_parent: string;
}

type PrepareResult = { status: "ready" } | ({ status: "missing" } & MissingSessionDirectory);
type MissingDirectoryAction = "switch" | "create";

export function useAgentMissingDirectory(sessionId: string | null) {
  const [missingDirectory, setMissingDirectory] = useState<MissingSessionDirectory | null>(null);
  const [resolving, setResolving] = useState(false);
  const pendingRef = useRef<(() => Promise<void>) | null>(null);

  const runOrDefer = useCallback(async (workingDir: string | undefined, run: () => Promise<void>) => {
    if (!sessionId || pendingRef.current) return;
    let result: PrepareResult;
    try {
      result = await invoke<PrepareResult>("prepare_agent_send", {
        id: sessionId,
        workingDir: workingDir ?? null,
      });
    } catch {
      showToast(i18n.t("missingDirectory.error"), "error");
      return;
    }
    if (result.status === "missing") {
      pendingRef.current = run;
      setMissingDirectory({
        missing_path: result.missing_path,
        nearest_parent: result.nearest_parent,
      });
      return;
    }
    await run();
  }, [sessionId]);

  const resolve = useCallback(async (action: MissingDirectoryAction) => {
    if (!sessionId || !missingDirectory || resolving) return;
    setResolving(true);
    try {
      await invoke("resolve_missing_session_directory", {
        id: sessionId,
        missingPath: missingDirectory.missing_path,
        action,
      });
      const pending = pendingRef.current;
      pendingRef.current = null;
      setMissingDirectory(null);
      if (pending) await pending();
    } catch {
      showToast(i18n.t("missingDirectory.error"), "error");
    } finally {
      setResolving(false);
    }
  }, [missingDirectory, resolving, sessionId]);

  return { missingDirectory, resolving, runOrDefer, resolve };
}
