import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useAgentStream } from "./use-agent-stream";
import { buildSegmentedMessage } from "./agent-chat-utils";
import type { ToolActivity, StreamSegment } from "./agent-chat-utils";
import type { AgentMessage, AgentSession } from "@/types/agent";

interface ChatState {
  messages: AgentMessage[];
  completedSegments: StreamSegment[];
  currentContent: string;
  currentThinking: string;
  currentTools: ToolActivity[];
  isStreaming: boolean;
  tps: number;
  tokenCount: number;
  lastRequestTokens: number;
}

export function useAgentChat(
  sessionId: string | null,
  model: string,
  provider: string,
  onPermissionRequest?: (id: string, toolName: string, args: Record<string, unknown>) => void,
) {
  const [state, setState] = useState<ChatState>({
    messages: [], completedSegments: [],
    currentContent: "", currentThinking: "", currentTools: [],
    isStreaming: false, tps: 0, tokenCount: 0, lastRequestTokens: 0,
  });
  const skillRef = useRef<string | null>(null);
  const savingRef = useRef(false);
  const { startStream, stopStream } = useAgentStream();

  useEffect(() => {
    if (!sessionId) return;
    const loadSession = () => {
      invoke<AgentSession>("get_agent_session", { id: sessionId })
        .then((session) => setState((s) => ({
          ...s, messages: session.messages, tokenCount: session.accumulated_tokens,
        })))
        .catch((e: unknown) => console.warn("Session load:", e));
    };
    loadSession();

    const unlisten = listen<{ session_id: string }>("wakeup-completed", (e) => {
      if (e.payload?.session_id === sessionId) {
        loadSession();
      }
    });
    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [sessionId]);

  const buildMessages = useCallback((msgs: AgentMessage[]): AgentMessage[] => {
    if (!skillRef.current) return msgs;
    const sys: AgentMessage = {
      id: "system-skill", role: "user", content: skillRef.current,
      files: [], timestamp: new Date().toISOString(),
    };
    return [sys, ...msgs];
  }, []);

  const doStream = useCallback(async (msgs: AgentMessage[], workingDir?: string) => {
    if (!sessionId) return;
    setState((s) => ({
      ...s, messages: msgs, completedSegments: [],
      currentContent: "", currentThinking: "", currentTools: [],
      isStreaming: true, tps: 0,
    }));
    await startStream(sessionId, model, provider, buildMessages(msgs), [], true, {
      onToken: (content, _tc, tps) =>
        setState((s) => ({ ...s, currentContent: s.currentContent + content, tps })),
      onThinking: (content) =>
        setState((s) => ({ ...s, currentThinking: s.currentThinking + content })),
      onToolCall: (name, args) =>
        setState((s) => ({ ...s, currentTools: [...s.currentTools, { name, args }] })),
      onToolResult: (name, content, isError) => setState((s) => {
        const tools = [...s.currentTools];
        for (let i = tools.length - 1; i >= 0; i--) {
          if (tools[i].name === name && !tools[i].result) {
            tools[i] = { ...tools[i], result: content, isError };
            break;
          }
        }
        return { ...s, currentTools: tools };
      }),
      onPermissionRequest: (id, toolName, args) => onPermissionRequest?.(id, toolName, args),
      onTurnEnd: () => setState((s) => ({
        ...s,
        completedSegments: [...s.completedSegments, {
          thinking: s.currentThinking, tools: s.currentTools, content: s.currentContent,
        }],
        currentContent: "", currentThinking: "", currentTools: [],
      })),
      onDone: (evalCount, finalTps, promptTokens) => setState((s) => {
        const all = [...s.completedSegments];
        if (s.currentContent || s.currentThinking || s.currentTools.length > 0) {
          all.push({ thinking: s.currentThinking, tools: s.currentTools, content: s.currentContent });
        }
        const tokens = (evalCount || 0) + (promptTokens || 0);
        if (all.length === 0) {
          return {
            ...s, isStreaming: false, tps: finalTps,
            tokenCount: s.tokenCount + tokens, lastRequestTokens: tokens,
          };
        }

        const built = buildSegmentedMessage(all);
        const msg: AgentMessage = {
          id: crypto.randomUUID(), role: "assistant",
          content: built.content, thinking: built.thinking,
          tool_activities: built.toolRecords, segments: built.segments,
          files: [], timestamp: new Date().toISOString(),
        };
        if (!savingRef.current && sessionId) {
          savingRef.current = true;
          invoke("add_messages_to_session", { id: sessionId, messages: [msg], tokens })
            .catch((e: unknown) => console.error("Save assistant msg:", e))
            .finally(() => { savingRef.current = false; });
        }
        return {
          ...s, messages: [...s.messages, msg], completedSegments: [],
          currentContent: "", currentThinking: "", currentTools: [],
          isStreaming: false, tps: finalTps,
          tokenCount: s.tokenCount + tokens, lastRequestTokens: tokens,
        };
      }),
      onError: (msg) => { setState((s) => ({ ...s, isStreaming: false })); console.error("Stream:", msg); },
    }, workingDir);
  }, [sessionId, model, provider, startStream, buildMessages, onPermissionRequest]);

  const sendMessage = useCallback(async (
    text: string,
    sentFiles?: { name: string; path?: string; preview?: string }[],
    workingDir?: string,
    projectId?: string,
  ) => {
    if (!sessionId) return;
    if (!text.trim() && (!sentFiles || sentFiles.length < 1)) return;
    while (savingRef.current) {
      await new Promise((r) => setTimeout(r, 50));
    }
    if (projectId && state.messages.length === 0) {
      const session = await invoke<Record<string, unknown>>("get_agent_session", { id: sessionId });
      if (!session.project_id) {
        session.project_id = projectId;
        await invoke("save_agent_session", { session }).catch(() => {});
      }
    }
    const files = (sentFiles ?? []).map((f) => ({
      name: f.name, path: f.path ?? "", mime_type: "", size: 0, thumbnail: f.preview,
    }));
    const userMsg: AgentMessage = {
      id: crypto.randomUUID(), role: "user", content: text,
      files, timestamp: new Date().toISOString(),
    };
    await invoke("add_messages_to_session", { id: sessionId, messages: [userMsg], tokens: 0 })
      .catch((e: unknown) => console.error("Save user msg:", e));
    await doStream([...state.messages, userMsg], workingDir);
  }, [sessionId, state.messages, doStream]);

  const syncTokenCount = useCallback(async () => {
    if (!sessionId) return;
    try {
      const session = await invoke<AgentSession>("get_agent_session", { id: sessionId });
      setState((s) => ({ ...s, tokenCount: session.accumulated_tokens }));
    } catch (e: unknown) {
      console.warn("syncTokenCount:", e);
    }
  }, [sessionId]);

  const reload = useCallback(async (messageId: string) => {
    if (!sessionId) return;
    const idx = state.messages.findIndex((m) => m.id === messageId);
    if (idx < 0) return;
    await invoke("truncate_and_replace_at", {
      sessionId, messageId, replacement: null,
    }).catch((e: unknown) => console.error("Truncate:", e));
    await syncTokenCount();
    await doStream(state.messages.slice(0, idx + 1));
  }, [sessionId, state.messages, doStream, syncTokenCount]);

  const edit = useCallback(async (messageId: string, newContent: string) => {
    if (!sessionId) return;
    const idx = state.messages.findIndex((m) => m.id === messageId);
    if (idx < 0) return;
    const newMsg: AgentMessage = {
      id: crypto.randomUUID(), role: "user", content: newContent,
      files: [], timestamp: new Date().toISOString(),
    };
    await invoke("truncate_and_replace_at", {
      sessionId, messageId, replacement: newMsg,
    }).catch((e: unknown) => console.error("Truncate+replace:", e));
    await syncTokenCount();
    await doStream([...state.messages.slice(0, idx), newMsg]);
  }, [sessionId, state.messages, doStream, syncTokenCount]);

  const stop = useCallback(async () => {
    if (sessionId) await stopStream(sessionId);
    setState((s) => ({ ...s, isStreaming: false }));
  }, [sessionId, stopStream]);

  const setSkill = useCallback((content: string | null) => { skillRef.current = content; }, []);

  return { ...state, sendMessage, reload, edit, stop, setSkill };
}
