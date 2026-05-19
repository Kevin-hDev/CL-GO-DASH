import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAgentStream } from "./use-agent-stream";
import { listenGatewaySessionUpdates } from "./use-gateway-session-updates";
import { EMPTY_CHAT_STATE, type ChatState } from "./agent-chat-stream-callbacks";
import { resolveSessionTokenCount } from "./agent-token-estimate";
import { showToast } from "@/lib/toast-emitter";
import i18n from "@/i18n";
import type { AgentMessage, AgentSession } from "@/types/agent";

const MAX_DELIVERED_PERMISSIONS = 64;

export function useAgentChat(
  sessionId: string | null,
  model: string,
  provider: string,
  onPermissionRequest?: (id: string, toolName: string, args: Record<string, unknown>) => void,
  supportsTools?: boolean,
  supportsThinking?: boolean,
  permissionMode?: string,
) {
  const [state, setState] = useState<ChatState>(EMPTY_CHAT_STATE);
  const [sessionLoading, setSessionLoading] = useState(true);
  const savingRef = useRef(false);
  const sessionRef = useRef(sessionId);
  const deliveredPermissionsRef = useRef<Set<string>>(new Set());
  const permissionRequestRef = useRef(onPermissionRequest);
  const { startStream, stopStream, subscribeToStream, getStreamSnapshot } = useAgentStream();
  // eslint-disable-next-line react-hooks/refs -- callback capture pattern for stable closures
  sessionRef.current = sessionId;
  const thinkingRef = useRef(supportsThinking);
  // eslint-disable-next-line react-hooks/refs -- callback capture pattern for stable closures
  thinkingRef.current = supportsThinking;
  // eslint-disable-next-line react-hooks/refs -- callback capture pattern for stable closures
  permissionRequestRef.current = onPermissionRequest;
  const permModeRef = useRef(permissionMode);
  // eslint-disable-next-line react-hooks/refs -- callback capture pattern for stable closures
  permModeRef.current = permissionMode;
  const sessionWorkingDirRef = useRef<string | undefined>(undefined);

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
    // eslint-disable-next-line react-hooks/set-state-in-effect -- reset on session change + fetch→setState are intentional
    setSessionLoading(true);
    setState(EMPTY_CHAT_STATE);
    deliveredPermissionsRef.current.clear();
    if (!sessionId) return;

    let alive = true;
    const applySnapshot = (snapshot: ReturnType<typeof getStreamSnapshot>) => {
      if (!snapshot || !alive || sessionRef.current !== sessionId) return;
      const { pendingPermissions, completed: _completed, ...chatState } = snapshot;
      setState(chatState);
      setSessionLoading(false);
      for (const request of pendingPermissions) {
        deliverPermission(request.id, request.toolName, request.arguments);
      }
    };

    const unsubscribe = subscribeToStream(sessionId, applySnapshot);
    applySnapshot(getStreamSnapshot(sessionId));

    invoke<AgentSession>("get_agent_session", { id: sessionId })
      .then((session) => {
        if (!alive || sessionRef.current !== sessionId) return;
        sessionWorkingDirRef.current = session.working_dir;
        const snapshot = getStreamSnapshot(sessionId);
        if (snapshot && snapshot.messages.length >= session.messages.length) {
          applySnapshot(snapshot);
          return;
        }
        setState((s) => ({
          ...s,
          messages: session.messages,
          tokenCount: resolveSessionTokenCount(session),
        }));
        setSessionLoading(false);
      })
      .catch((e: unknown) => { console.warn("Session load:", e); setSessionLoading(false); });

    const stopGatewayListener = listenGatewaySessionUpdates(sessionId, sessionRef, (session) => {
      setState((s) => ({ ...s, messages: session.messages, tokenCount: resolveSessionTokenCount(session) }));
    });
    return () => {
      alive = false;
      unsubscribe();
      stopGatewayListener();
    };
  }, [sessionId, subscribeToStream, getStreamSnapshot, deliverPermission]);

  const doStream = useCallback(async (
    llmMsgs: AgentMessage[],
    displayMsgs: AgentMessage[],
    streamSession: string,
    workingDir?: string,
    baseTokenCountOverride?: number,
    permissionMode?: string,
  ) => {
    await startStream(
      streamSession,
      model,
      provider,
      llmMsgs,
      thinkingRef.current ?? false,
      { displayMessages: displayMsgs, baseTokenCount: baseTokenCountOverride ?? state.tokenCount },
      workingDir,
      supportsTools,
      thinkingRef.current,
      permissionMode,
    );
  }, [model, provider, startStream, state.tokenCount, supportsTools]);

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
      if (!session.project_id) { session.project_id = projectId; await invoke("save_agent_session", { session }).catch(() => showToast(i18n.t("errors.sessionSaveFailed"), "error")); }
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
    await invoke("add_messages_to_session", { id: sessionId, messages: [userMsg], tokens: 0 }).catch(() => showToast(i18n.t("errors.sessionSaveFailed"), "error"));
    await doStream(llmMsgs, displayMsgs, sessionId, workingDir, undefined, permModeRef.current);
  }, [sessionId, state.messages, doStream]);

  const syncTokenCount = useCallback(async (): Promise<number> => {
    if (!sessionId) return state.tokenCount;
    const session = await invoke<AgentSession>("get_agent_session", { id: sessionId }).catch(() => null);
    if (session) {
      sessionWorkingDirRef.current = session.working_dir || sessionWorkingDirRef.current;
      setState((s) => ({ ...s, tokenCount: session.accumulated_tokens }));
      return session.accumulated_tokens;
    }
    return state.tokenCount;
  }, [sessionId, state.tokenCount]);

  const reload = useCallback(async (messageId: string) => {
    if (!sessionId) return;
    const idx = state.messages.findIndex((m) => m.id === messageId);
    if (idx < 0) return;
    await invoke("truncate_and_replace_at", { sessionId, messageId, replacement: null }).catch(() => showToast(i18n.t("errors.sessionSaveFailed"), "error"));
    const freshTokenCount = await syncTokenCount();
    const msgs = state.messages.slice(0, idx + 1);
    await doStream(msgs, msgs, sessionId, sessionWorkingDirRef.current, freshTokenCount, permModeRef.current);
  }, [sessionId, state.messages, doStream, syncTokenCount]);

  const edit = useCallback(async (messageId: string, newContent: string) => {
    if (!sessionId) return;
    const idx = state.messages.findIndex((m) => m.id === messageId);
    if (idx < 0) return;
    const newMsg: AgentMessage = { id: crypto.randomUUID(), role: "user", content: newContent, files: [], timestamp: new Date().toISOString() };
    await invoke("truncate_and_replace_at", { sessionId, messageId, replacement: newMsg }).catch(() => showToast(i18n.t("errors.sessionSaveFailed"), "error"));
    const freshTokenCount = await syncTokenCount();
    const msgs = [...state.messages.slice(0, idx), newMsg];
    await doStream(msgs, msgs, sessionId, sessionWorkingDirRef.current, freshTokenCount, permModeRef.current);
  }, [sessionId, state.messages, doStream, syncTokenCount]);

  const stop = useCallback(async () => {
    if (sessionId) await stopStream(sessionId);
    setState((s) => ({ ...s, isStreaming: false }));
  }, [sessionId, stopStream]);

  const ready = state.messages.length > 0 || !sessionId;

  return { ...state, ready, sessionLoading, sendMessage, reload, edit, stop };
}
