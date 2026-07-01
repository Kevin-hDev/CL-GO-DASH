import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import {
  activeTodoRuns,
  childSubagents,
  summarizeLastRequestChanges,
} from "@/lib/session-summary";
import type { AgentSession, AgentSessionMeta, StreamEvent } from "@/types/agent";

interface StreamEnvelope {
  sessionId: string;
  event: StreamEvent;
}

const REFRESH_EVENTS = new Set<StreamEvent["event"]>([
  "done",
  "todoUpdated",
  "planPreviewUpdated",
  "planModeUpdated",
  "subagentSpawned",
  "subagentCompleted",
  "compressionComplete",
]);

export function useSessionSummary(sessionId: string | null) {
  const [session, setSession] = useState<AgentSession | null>(null);
  const [subagentSessions, setSubagentSessions] = useState<AgentSessionMeta[]>([]);
  const timerRef = useRef<number | null>(null);
  const requestSeqRef = useRef(0);

  const refresh = useCallback(async () => {
    const requestSeq = requestSeqRef.current + 1;
    requestSeqRef.current = requestSeq;
    if (!sessionId) {
      setSession(null);
      setSubagentSessions([]);
      return;
    }
    try {
      const [nextSession, children] = await Promise.all([
        invoke<AgentSession>("get_agent_session", { id: sessionId }),
        invoke<AgentSessionMeta[]>("list_subagents", { parentSessionId: sessionId, runId: null }),
      ]);
      if (requestSeqRef.current !== requestSeq) return;
      setSession(nextSession);
      setSubagentSessions(children);
    } catch {
      if (requestSeqRef.current !== requestSeq) return;
      setSession(null);
      setSubagentSessions([]);
    }
  }, [sessionId]);

  const scheduleRefresh = useCallback((delayMs = 0) => {
    if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    timerRef.current = window.setTimeout(() => void refresh(), delayMs);
  }, [refresh]);

  useEffect(() => {
    let cancelled = false;
    queueMicrotask(() => {
      if (!cancelled) void refresh();
    });
    return () => {
      cancelled = true;
      requestSeqRef.current += 1;
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    };
  }, [refresh, sessionId]);

  useEffect(() => {
    if (!sessionId) return;
    const streamUnlisten = listen<StreamEnvelope>("agent-stream-event", (event) => {
      const payload = event.payload;
      if (payload.sessionId !== sessionId) return;
      if (!REFRESH_EVENTS.has(payload.event.event)) return;
      scheduleRefresh(payload.event.event === "done" ? 300 : 80);
    });
    const sessionUnlisten = listen("agent-session-updated", () => scheduleRefresh(80));
    return () => {
      cleanupTauriListener(streamUnlisten);
      cleanupTauriListener(sessionUnlisten);
    };
  }, [scheduleRefresh, sessionId]);

  return useMemo(() => ({
    session,
    activeTodos: activeTodoRuns(session),
    plans: session?.plan_runs ?? [],
    subagents: sessionId ? childSubagents(sessionId, subagentSessions) : [],
    changes: summarizeLastRequestChanges(session?.messages ?? []),
  }), [session, sessionId, subagentSessions]);
}

export type SessionSummaryHookState = ReturnType<typeof useSessionSummary>;
