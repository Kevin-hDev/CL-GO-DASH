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
  supportsTools?: boolean,
  supportsThinking?: boolean,
) {
  const [state, setState] = useState<ChatState>(EMPTY_CHAT_STATE);
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
      const { pendingPermissions, completed, ...chatState } = snapshot;
      void completed;
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

  const doStream = useCallback(async (
    llmMsgs: AgentMessage[],
    displayMsgs: AgentMessage[],
    streamSession: string,
    workingDir?: string,
    baseTokenCountOverride?: number,
  ) => {
    await startStream(
      streamSession,
      model,
      provider,
      llmMsgs,
      [],
      true,
      { displayMessages: displayMsgs, baseTokenCount: baseTokenCountOverride ?? state.tokenCount },
      workingDir,
      supportsTools,
      supportsThinking,
    );
  }, [model, provider, startStream, state.tokenCount, supportsTools, supportsThinking]);

  const sendMessage = useCallback(async (
    text: string,
    sentFiles?: { name: string; path?: string; preview?: string }[],
    workingDir?: string,
    projectId?: string,
    skills?: { name: string; content: string }[],
  ) => {
    const hasText = !!text.trim();
    const hasFiles = !!sentFiles && sentFiles.length > 0;
    const hasSkill = !!skills && skills.length > 0;
    if (!sessionId || (!hasText && !hasFiles && !hasSkill)) return;
    while (savingRef.current) await new Promise((r) => setTimeout(r, 50));
    if (projectId && state.messages.length === 0) {
      const session = await invoke<Record<string, unknown>>("get_agent_session", { id: sessionId });
      if (!session.project_id) { session.project_id = projectId; await invoke("save_agent_session", { session }).catch((e: unknown) => console.error("Save session:", e)); }
    }
    const files = (sentFiles ?? []).map((f) => ({ name: f.name, path: f.path ?? "", mime_type: "", size: 0, thumbnail: f.preview }));
    const skillNames = hasSkill ? skills.map((s) => s.name) : undefined;
    const userMsg: AgentMessage = {
      id: crypto.randomUUID(), role: "user", content: text || "",
      files, timestamp: new Date().toISOString(),
      skill_names: skillNames,
    };
    const displayMsgs = [...state.messages, userMsg];
    const llmMsgs = [...state.messages];
    if (hasSkill) {
      for (const s of skills) {
        llmMsgs.push({ id: "skill-" + crypto.randomUUID(), role: "user", content: `The user has loaded the following skill. Follow its instructions exactly:\n\n${s.content}`, files: [], timestamp: new Date().toISOString() });
      }
    }
    llmMsgs.push(userMsg);
    await invoke("add_messages_to_session", { id: sessionId, messages: [userMsg], tokens: 0 }).catch((e: unknown) => console.error("Save user msg:", e));
    await doStream(llmMsgs, displayMsgs, sessionId, workingDir);
  }, [sessionId, state.messages, doStream]);

  const syncTokenCount = useCallback(async (): Promise<number> => {
    if (!sessionId) return state.tokenCount;
    const session = await invoke<AgentSession>("get_agent_session", { id: sessionId }).catch(() => null);
    if (session) {
      setState((s) => ({ ...s, tokenCount: session.accumulated_tokens }));
      return session.accumulated_tokens;
    }
    return state.tokenCount;
  }, [sessionId, state.tokenCount]);

  const reload = useCallback(async (messageId: string) => {
    if (!sessionId) return;
    const idx = state.messages.findIndex((m) => m.id === messageId);
    if (idx < 0) return;
    await invoke("truncate_and_replace_at", { sessionId, messageId, replacement: null }).catch((e: unknown) => console.error("Truncate:", e));
    const freshTokenCount = await syncTokenCount();
    const msgs = state.messages.slice(0, idx + 1);
    await doStream(msgs, msgs, sessionId, undefined, freshTokenCount);
  }, [sessionId, state.messages, doStream, syncTokenCount]);

  const edit = useCallback(async (messageId: string, newContent: string) => {
    if (!sessionId) return;
    const idx = state.messages.findIndex((m) => m.id === messageId);
    if (idx < 0) return;
    const newMsg: AgentMessage = { id: crypto.randomUUID(), role: "user", content: newContent, files: [], timestamp: new Date().toISOString() };
    await invoke("truncate_and_replace_at", { sessionId, messageId, replacement: newMsg }).catch((e: unknown) => console.error("Truncate+replace:", e));
    const freshTokenCount = await syncTokenCount();
    const msgs = [...state.messages.slice(0, idx), newMsg];
    await doStream(msgs, msgs, sessionId, undefined, freshTokenCount);
  }, [sessionId, state.messages, doStream, syncTokenCount]);

  const stop = useCallback(async () => {
    if (sessionId) await stopStream(sessionId);
    setState((s) => ({ ...s, isStreaming: false }));
  }, [sessionId, stopStream]);

  const ready = state.messages.length > 0 || !sessionId;

  return { ...state, ready, sendMessage, reload, edit, stop };
}
