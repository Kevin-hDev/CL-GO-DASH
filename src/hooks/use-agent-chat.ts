import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useAgentStream } from "./use-agent-stream";
import type { AgentMessage, AgentSession } from "@/types/agent";

interface ChatState {
  messages: AgentMessage[];
  streamingContent: string;
  streamingThinking: string;
  isStreaming: boolean;
  tps: number;
  tokenCount: number;
}

export function useAgentChat(sessionId: string | null, model: string) {
  const [state, setState] = useState<ChatState>({
    messages: [], streamingContent: "", streamingThinking: "",
    isStreaming: false, tps: 0, tokenCount: 0,
  });
  const skillRef = useRef<string | null>(null);
  const { startStream, stopStream } = useAgentStream();

  useEffect(() => {
    if (!sessionId) return;
    invoke<AgentSession>("get_agent_session", { id: sessionId })
      .then((session) => {
        setState((s) => ({
          ...s, messages: session.messages,
          tokenCount: session.accumulated_tokens,
        }));
      })
      .catch((e: unknown) => console.warn("Session load:", e));
  }, [sessionId]);

  const buildMessages = useCallback((msgs: AgentMessage[]): AgentMessage[] => {
    if (!skillRef.current) return msgs;
    const systemMsg: AgentMessage = {
      id: "system-skill", role: "user", content: skillRef.current,
      files: [], timestamp: new Date().toISOString(),
    };
    return [systemMsg, ...msgs];
  }, []);

  const doStream = useCallback(async (msgs: AgentMessage[]) => {
    if (!sessionId) return;
    setState((s) => ({
      ...s, messages: msgs, streamingContent: "", streamingThinking: "",
      isStreaming: true, tps: 0,
    }));
    const toSend = buildMessages(msgs);
    await startStream(sessionId, model, toSend, [], false, {
      onToken: (content, _tc, tps) => {
        setState((s) => ({
          ...s, streamingContent: s.streamingContent + content, tps,
        }));
      },
      onThinking: (content) => {
        setState((s) => ({
          ...s, streamingThinking: s.streamingThinking + content,
        }));
      },
      onToolCall: () => {},
      onToolResult: () => {},
      onDone: (evalCount, finalTps, promptTokens) => {
        setState((s) => {
          const assistantMsg: AgentMessage = {
            id: crypto.randomUUID(), role: "assistant",
            content: s.streamingContent,
            thinking: s.streamingThinking || undefined,
            files: [], timestamp: new Date().toISOString(),
          };
          const addedTokens = evalCount + promptTokens;
          if (sessionId) {
            invoke("add_messages_to_session", {
              id: sessionId,
              messages: [assistantMsg],
              tokens: addedTokens,
            }).catch((e: unknown) => console.error("Save assistant msg:", e));
          }
          return {
            ...s, messages: [...s.messages, assistantMsg],
            streamingContent: "", streamingThinking: "",
            isStreaming: false, tps: finalTps,
            tokenCount: s.tokenCount + addedTokens,
          };
        });
      },
      onError: (message) => {
        setState((s) => ({ ...s, isStreaming: false }));
        console.error("Stream error:", message);
      },
    });
  }, [sessionId, model, startStream, buildMessages]);

  const sendMessage = useCallback(async (text: string) => {
    if (!sessionId || !text.trim()) return;
    const userMsg: AgentMessage = {
      id: crypto.randomUUID(), role: "user", content: text,
      files: [], timestamp: new Date().toISOString(),
    };
    // Sauvegarder le message user
    invoke("add_messages_to_session", {
      id: sessionId, messages: [userMsg], tokens: 0,
    }).catch((e: unknown) => console.error("Save user msg:", e));
    await doStream([...state.messages, userMsg]);
  }, [sessionId, state.messages, doStream]);

  const reload = useCallback(async (messageId: string) => {
    if (!sessionId) return;
    const idx = state.messages.findIndex((m) => m.id.localeCompare(messageId) === 0);
    if (idx < 0) return;
    await invoke("truncate_session_at", { sessionId, messageId }).catch(() => {});
    await doStream(state.messages.slice(0, idx + 1));
  }, [sessionId, state.messages, doStream]);

  const edit = useCallback(async (messageId: string, newContent: string) => {
    if (!sessionId) return;
    const idx = state.messages.findIndex((m) => m.id.localeCompare(messageId) === 0);
    if (idx < 0) return;
    await invoke("truncate_session_at", { sessionId, messageId }).catch(() => {});
    const newMsg: AgentMessage = {
      id: crypto.randomUUID(), role: "user", content: newContent,
      files: [], timestamp: new Date().toISOString(),
    };
    await doStream([...state.messages.slice(0, idx), newMsg]);
  }, [sessionId, state.messages, doStream]);

  const stop = useCallback(async () => {
    if (sessionId) await stopStream(sessionId);
    setState((s) => ({ ...s, isStreaming: false }));
  }, [sessionId, stopStream]);

  const setSkill = useCallback((content: string | null) => {
    skillRef.current = content;
  }, []);

  return { ...state, sendMessage, reload, edit, stop, setSkill };
}
