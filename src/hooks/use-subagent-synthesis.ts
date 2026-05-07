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
    console.log(`[DIAG:synthesis] check: allDone=${allDone} isStreaming=${isStreaming} key=${key?.slice(0,8)} sentRef=${sentRunRef.current?.slice(0,8) ?? "null"}`);
    if (!allDone) {
      return;
    }
    if (isStreaming) {
      console.log(`[DIAG:synthesis] BLOCKED by isStreaming=true`);
      return;
    }
    if (sentRunRef.current === key) {
      console.log(`[DIAG:synthesis] SKIP already sent for key=${key?.slice(0,8)}`);
      return;
    }

    console.log(`[DIAG:synthesis] FIRING synthesize_subagent_results for parent=${parentSessionId.slice(0,8)} runId=${runId?.slice(0,8) ?? "null"}`);
    sentRunRef.current = key;
    void invoke("synthesize_subagent_results", {
      parentSessionId,
      runId: runId ?? null,
    }).then(() => {
      console.log(`[DIAG:synthesis] invoke OK`);
      onStarted?.();
    }).catch((e) => {
      console.error(`[DIAG:synthesis] invoke FAILED:`, e);
      sentRunRef.current = null;
    });
  }, [allDone, isStreaming, onStarted, parentSessionId, runId]);
}
