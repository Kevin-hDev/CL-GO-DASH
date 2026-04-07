import { useRef, useCallback } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import type { StreamEvent, AgentMessage } from "@/types/agent";

interface StreamCallbacks {
  onToken: (content: string, tokenCount: number, tps: number) => void;
  onThinking: (content: string) => void;
  onToolCall: (name: string, args: Record<string, unknown>) => void;
  onToolResult: (name: string, content: string, isError: boolean) => void;
  onDone: (evalCount: number, finalTps: number, promptTokens: number) => void;
  onError: (message: string) => void;
}

export function useAgentStream() {
  const streamingRef = useRef(false);

  const startStream = useCallback(async (
    sessionId: string,
    model: string,
    messages: AgentMessage[],
    tools: unknown[],
    think: boolean,
    callbacks: StreamCallbacks,
  ) => {
    streamingRef.current = true;

    const channel = new Channel<StreamEvent>();
    channel.onmessage = (event: StreamEvent) => {
      switch (event.event) {
        case "token":
          callbacks.onToken(event.data.content, event.data.tokenCount, event.data.tps);
          break;
        case "thinking":
          callbacks.onThinking(event.data.content);
          break;
        case "toolCall":
          callbacks.onToolCall(event.data.name, event.data.arguments);
          break;
        case "toolResult":
          callbacks.onToolResult(event.data.name, event.data.content, event.data.isError);
          break;
        case "done":
          callbacks.onDone(event.data.evalCount, event.data.finalTps, event.data.promptTokens);
          streamingRef.current = false;
          break;
        case "error":
          callbacks.onError(event.data.message);
          streamingRef.current = false;
          break;
      }
    };

    const chatMessages = messages.map((m) => ({
      role: m.role,
      content: m.content,
      images: null,
      tool_calls: m.tool_calls ?? null,
      tool_name: m.tool_name ?? null,
    }));

    await invoke("chat_stream", {
      sessionId,
      model,
      messages: chatMessages,
      tools,
      think,
      onEvent: channel,
    });
  }, []);

  const stopStream = useCallback(async (sessionId: string) => {
    await invoke("cancel_agent_request", { sessionId });
    streamingRef.current = false;
  }, []);

  return { startStream, stopStream, isStreaming: () => streamingRef.current };
}
