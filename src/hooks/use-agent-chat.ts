import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useAgentStream } from "./use-agent-stream";
import { EMPTY_CHAT_STATE, type ChatState } from "./agent-chat-stream-callbacks";
import type { AgentMessage, AgentSession } from "@/types/agent";

const MAX_DELIVERED_PERMISSIONS = 64;

export function useAgentChat(
  sessionId: string | null,
  model: string,
  provider: string,
  onPermissionRequest?: (id: string, toolName: string, args: Record<string, unknown>) => void,
) {
  const [state, setState] = useState<ChatState>(EMPTY_CHAT_STATE);
  const skillRef = useRef<string | null>(null);
  const savingRef = useRef(false);
  const sessionRef = useRef(sessionId);
  const deliveredPermissionsRef = useRef<Set<string>>(new Set());
  const permissionRequestRef = useRef(onPermissionRequest);
  const { startStream, stopStream, subscribeToStream, getStreamSnapshot } = useAgentStream();

  sessionRef.current = sessionId;
  permissionRequestRef.current = onPermissionRequest;

  const deliverPermission = useCallback((id: string, toolName: string, args: Record<string, unknown>) => {
    const delivered = deliveredPermissionsRef.current;
    if (delivered.has(id)) return;
    delivered.add(id);
    while (delivered.size > MAX_DELIVERED_PERMISSIONS) {
      const first = delivered.values().next().value;
      if (!first) break;
      delivered.delete(first);
    }
    permissionRequestRef.current?.(id, toolName, args);
  }, []);

  useEffect(() => {
    setState(EMPTY_CHAT_STATE);
    deliveredPermissionsRef.current.clear();
    if (!sessionId) return;

    let alive = true;
    const applySnapshot = (snapshot: ReturnType<typeof getStreamSnapshot>) => {
      if (!snapshot || !alive || sessionRef.current !== sessionId) return;
      const { pendingPermissions, completed, error, ...chatState } = snapshot;
      void completed;
      void error;
      setState(chatState);
      for (const request of pendingPermissions) {
        deliverPermission(request.id, request.toolName, request.arguments);
      }
    };

    const unsubscribe = subscribeToStream(sessionId, applySnapshot);
    applySnapshot(getStreamSnapshot(sessionId));

    invoke<AgentSession>("get_agent_session", { id: sessionId })
      .then((session) => {
        if (!alive || sessionRef.current !== sessionId) return;
        const snapshot = getStreamSnapshot(sessionId);
        if (snapshot) {
          applySnapshot(snapshot);
          return;
        }
        setState((s) => ({
          ...s,
          messages: session.messages,
          tokenCount: session.accumulated_tokens,
        }));
      })
      .catch((e: unknown) => console.warn("Session load:", e));

    const unlisten = listen<{ session_id: string }>("wakeup-completed", (e) => {
      if (e.payload?.session_id === sessionId && sessionRef.current === sessionId) {
        invoke<AgentSession>("get_agent_session", { id: sessionId })
          .then((session) => setState((s) => ({ ...s, messages: session.messages, tokenCount: session.accumulated_tokens })))
          .catch((e: unknown) => console.warn("Session reload:", e));
      }
    });
    return () => {
      alive = false;
      unsubscribe();
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [sessionId, subscribeToStream, getStreamSnapshot, deliverPermission]);

  const buildMessages = useCallback((msgs: AgentMessage[]): AgentMessage[] => {
    if (!skillRef.current) return msgs;
    return [{ id: "system-skill", role: "user", content: skillRef.current, files: [], timestamp: new Date().toISOString() }, ...msgs];
  }, []);

  const doStream = useCallback(async (msgs: AgentMessage[], streamSession: string, workingDir?: string) => {
    await startStream(
      streamSession,
      model,
      provider,
      buildMessages(msgs),
      [],
      true,
      { displayMessages: msgs, baseTokenCount: state.tokenCount },
      workingDir,
    );
  }, [model, provider, startStream, buildMessages, state.tokenCount]);

  const sendMessage = useCallback(async (
    text: string,
    sentFiles?: { name: string; path?: string; preview?: string }[],
    workingDir?: string,
    projectId?: string,
  ) => {
    if (!sessionId || (!text.trim() && (!sentFiles || sentFiles.length < 1))) return;
    while (savingRef.current) await new Promise((r) => setTimeout(r, 50));
    if (projectId && state.messages.length === 0) {
      const session = await invoke<Record<string, unknown>>("get_agent_session", { id: sessionId });
      if (!session.project_id) { session.project_id = projectId; await invoke("save_agent_session", { session }).catch(() => {}); }
    }
    const files = (sentFiles ?? []).map((f) => ({ name: f.name, path: f.path ?? "", mime_type: "", size: 0, thumbnail: f.preview }));
    const userMsg: AgentMessage = { id: crypto.randomUUID(), role: "user", content: text, files, timestamp: new Date().toISOString() };
    await invoke("add_messages_to_session", { id: sessionId, messages: [userMsg], tokens: 0 }).catch((e: unknown) => console.error("Save user msg:", e));
    await doStream([...state.messages, userMsg], sessionId, workingDir);
  }, [sessionId, state.messages, doStream]);

  const syncTokenCount = useCallback(async () => {
    if (!sessionId) return;
    const session = await invoke<AgentSession>("get_agent_session", { id: sessionId }).catch(() => null);
    if (session) setState((s) => ({ ...s, tokenCount: session.accumulated_tokens }));
  }, [sessionId]);

  const reload = useCallback(async (messageId: string) => {
    if (!sessionId) return;
    const idx = state.messages.findIndex((m) => m.id === messageId);
    if (idx < 0) return;
    await invoke("truncate_and_replace_at", { sessionId, messageId, replacement: null }).catch((e: unknown) => console.error("Truncate:", e));
    await syncTokenCount();
    await doStream(state.messages.slice(0, idx + 1), sessionId);
  }, [sessionId, state.messages, doStream, syncTokenCount]);

  const edit = useCallback(async (messageId: string, newContent: string) => {
    if (!sessionId) return;
    const idx = state.messages.findIndex((m) => m.id === messageId);
    if (idx < 0) return;
    const newMsg: AgentMessage = { id: crypto.randomUUID(), role: "user", content: newContent, files: [], timestamp: new Date().toISOString() };
    await invoke("truncate_and_replace_at", { sessionId, messageId, replacement: newMsg }).catch((e: unknown) => console.error("Truncate+replace:", e));
    await syncTokenCount();
    await doStream([...state.messages.slice(0, idx), newMsg], sessionId);
  }, [sessionId, state.messages, doStream, syncTokenCount]);

  const stop = useCallback(async () => {
    if (sessionId) await stopStream(sessionId);
    setState((s) => ({ ...s, isStreaming: false }));
  }, [sessionId, stopStream]);

  const setSkill = useCallback((content: string | null) => { skillRef.current = content; }, []);
  const ready = state.messages.length > 0 || !sessionId;

  return { ...state, ready, sendMessage, reload, edit, stop, setSkill };
}
