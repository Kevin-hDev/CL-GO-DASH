import { useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

interface SubagentSynthesisOptions {
  parentSessionId: string;
  allDone: boolean;
  runId?: string;
  isStreaming: boolean;
  onStarted?: () => void;
}

export function useSubagentSynthesis({
  parentSessionId,
  allDone,
  runId,
  isStreaming,
  onStarted,
}: SubagentSynthesisOptions) {
  const sentRunRef = useRef<string | null>(null);

  useEffect(() => {
    const key = runId ?? parentSessionId;
    if (!allDone) {
      return;
    }
    if (isStreaming) {
      return;
    }
    if (sentRunRef.current === key) {
      return;
    }

    sentRunRef.current = key;
    void invoke("synthesize_subagent_results", {
      parentSessionId,
      runId: runId ?? null,
    }).then(() => {
      onStarted?.();
    }).catch(() => {
      sentRunRef.current = null;
    });
  }, [allDone, isStreaming, onStarted, parentSessionId, runId]);
}
