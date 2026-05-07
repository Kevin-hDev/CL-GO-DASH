import { useState, useEffect, useCallback, useRef } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { AgentSessionMeta, SubagentInfo, StreamEvent } from "@/types/agent";

interface StreamEnvelope {
  sessionId: string;
  event: StreamEvent;
}

interface StoreEntry {
  active: SubagentInfo[];
  completed: SubagentInfo[];
  allDone: boolean;
  runId?: string;
}

const MAX_STORE_ENTRIES = 32;
const globalStore = new Map<string, StoreEntry>();

function evictGlobalStore() {
  while (globalStore.size > MAX_STORE_ENTRIES) {
    const oldest = globalStore.keys().next().value;
    if (oldest) globalStore.delete(oldest);
  }
}

let globalListenerPromise: Promise<void> | null = null;

function ensureGlobalListener() {
  if (globalListenerPromise) return globalListenerPromise;
  globalListenerPromise = listen<StreamEnvelope>("agent-stream-event", (event) => {
    const parentId = event.payload.sessionId;
    const e = event.payload.event;

    if (e.event === "subagentSpawned") {
      const runId = e.data.runId;
      const store = globalStore.get(parentId) ?? { active: [], completed: [], allDone: false, runId };
      console.log(`[DIAG:subagents] SPAWNED on parent=${parentId.slice(0,8)} runId=${runId?.slice(0,8) ?? "null"} storeRunId=${store.runId?.slice(0,8) ?? "null"} active=${store.active.length} completed=${store.completed.length}`);
      if (runId && store.runId && store.runId !== runId) {
        console.log(`[DIAG:subagents] NEW RUN detected → clearing old completed/active`);
        store.completed = [];
        store.active = [];
      }
      store.runId = runId;
      store.active = [
        ...store.active,
        {
          sessionId: e.data.subagentSessionId,
          name: e.data.subagentName,
          type: e.data.subagentType as "explorer" | "coder",
          status: "running",
          promptPreview: e.data.promptPreview,
          runId,
        },
      ];
      store.allDone = false;
      globalStore.set(parentId, store);
      evictGlobalStore();
    }

    if (e.event === "subagentCompleted") {
      const store = globalStore.get(parentId) ?? { active: [], completed: [], allDone: false };
      console.log(`[DIAG:subagents] COMPLETED on parent=${parentId.slice(0,8)} child=${e.data.subagentSessionId.slice(0,8)} allDone=${e.data.allDone} active=${store.active.length} completed=${store.completed.length}`);
      const found = store.active.find(
        (s) => s.sessionId === e.data.subagentSessionId,
      );
      store.active = store.active.filter(
        (s) => s.sessionId !== e.data.subagentSessionId,
      );
      store.completed = [
        ...store.completed,
        {
          sessionId: e.data.subagentSessionId,
          name: found?.name ?? "agent",
          type: found?.type ?? "explorer",
          status: e.data.status,
          promptPreview: found?.promptPreview ?? "",
          runId: found?.runId ?? store.runId,
        },
      ];
      store.allDone = e.data.allDone;
      globalStore.set(parentId, store);
      evictGlobalStore();
    }
  }).then(() => { /* listener active */ });
  return globalListenerPromise;
}

export function useSubagents(parentSessionId: string | undefined) {
  const [active, setActive] = useState<SubagentInfo[]>([]);
  const [completed, setCompleted] = useState<SubagentInfo[]>([]);
  const [allDone, setAllDone] = useState(false);
  const [doneRunId, setDoneRunId] = useState<string | undefined>(undefined);
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    if (!parentSessionId) {
      setActive([]);
      setCompleted([]);
      setAllDone(false);
      setDoneRunId(undefined);
      return;
    }

    void ensureGlobalListener();
    void invoke<AgentSessionMeta[]>("list_subagents", {
      parentSessionId,
      runId: null,
    }).then((items) => {
      const mapped = items.map((item) => ({
        sessionId: item.id,
        name: item.name,
        type: item.subagent_type ?? "explorer",
        status: item.subagent_status ?? "completed",
        promptPreview: "",
        runId: item.subagent_run_id,
      }));
      setCompleted(mapped.filter((item) => item.status !== "running"));
      setActive(mapped.filter((item) => item.status === "running"));
    }).catch(() => {});

    const sync = () => {
      const store = globalStore.get(parentSessionId);
      if (store) {
        setActive([...store.active]);
        setCompleted((prev) => {
          if (store.completed.length > 0 && store.completed.length !== prev.length) {
            return [...store.completed];
          }
          return prev;
        });
        if (store.allDone) {
          setAllDone(true);
          setDoneRunId(store.runId);
          store.allDone = false;
        }
      }
    };

    sync();
    intervalRef.current = setInterval(sync, 500);

    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, [parentSessionId]);

  const cancelSubagent = useCallback(
    async (sessionId: string) => {
      await invoke("cancel_subagent", { subagentSessionId: sessionId });
      if (parentSessionId) {
        const store = globalStore.get(parentSessionId);
        if (store) {
          store.active = store.active.filter((s) => s.sessionId !== sessionId);
        }
      }
      setActive((prev) => prev.filter((s) => s.sessionId !== sessionId));
    },
    [parentSessionId],
  );

  const clearAllDone = useCallback(() => setAllDone(false), []);
  const clearSynthesisSignal = useCallback(() => {
    setAllDone(false);
    setDoneRunId(undefined);
  }, []);

  return { active, completed, allDone, doneRunId, cancelSubagent, clearAllDone, clearSynthesisSignal };
}
