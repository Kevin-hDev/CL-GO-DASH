import { useState, useCallback } from "react";
import { useAgentStream } from "./use-agent-stream";
import type { AgentMessage } from "@/types/agent";

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
    messages: [],
    streamingContent: "",
    streamingThinking: "",
    isStreaming: false,
    tps: 0,
    tokenCount: 0,
  });

  const { startStream, stopStream } = useAgentStream();

  const sendMessage = useCallback(async (text: string) => {
    if (!sessionId || !text.trim()) return;

    const userMsg: AgentMessage = {
      id: crypto.randomUUID(),
      role: "user",
      content: text,
      files: [],
      timestamp: new Date().toISOString(),
    };

    setState((s) => ({
      ...s,
      messages: [...s.messages, userMsg],
      streamingContent: "",
      streamingThinking: "",
      isStreaming: true,
      tps: 0,
    }));

    await startStream(sessionId, model, [...state.messages, userMsg], [], false, {
      onToken: (content, tokenCount, tps) => {
        setState((s) => ({
          ...s,
          streamingContent: s.streamingContent + content,
          tps,
          tokenCount,
        }));
      },
      onThinking: (content) => {
        setState((s) => ({
          ...s,
          streamingThinking: s.streamingThinking + content,
        }));
      },
      onToolCall: () => {},
      onToolResult: () => {},
      onDone: (evalCount, finalTps, promptTokens) => {
        setState((s) => {
          const assistantMsg: AgentMessage = {
            id: crypto.randomUUID(),
            role: "assistant",
            content: s.streamingContent,
            thinking: s.streamingThinking || undefined,
            files: [],
            timestamp: new Date().toISOString(),
          };
          return {
            ...s,
            messages: [...s.messages, assistantMsg],
            streamingContent: "",
            streamingThinking: "",
            isStreaming: false,
            tps: finalTps,
            tokenCount: s.tokenCount + evalCount + promptTokens,
          };
        });
      },
      onError: (message) => {
        setState((s) => ({ ...s, isStreaming: false }));
        console.error("Stream error:", message);
      },
    });
  }, [sessionId, model, state.messages, startStream]);

  const stop = useCallback(async () => {
    if (sessionId) await stopStream(sessionId);
    setState((s) => ({ ...s, isStreaming: false }));
  }, [sessionId, stopStream]);

  return { ...state, sendMessage, stop };
}
