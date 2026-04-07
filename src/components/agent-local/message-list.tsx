import { useEffect, useRef } from "react";
import { UserMessage } from "./user-message";
import { AssistantMessage } from "./assistant-message";
import type { AgentMessage } from "@/types/agent";

interface MessageListProps {
  messages: AgentMessage[];
  streamingContent: string;
  streamingThinking: string;
  isStreaming: boolean;
}

export function MessageList({
  messages, streamingContent, streamingThinking, isStreaming,
}: MessageListProps) {
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [messages.length, streamingContent]);

  return (
    <div style={{ flex: 1, overflowY: "auto", padding: "var(--space-md) var(--space-lg)" }}>
      {messages.map((msg) => {
        if (msg.role === "user") {
          return <UserMessage key={msg.id} content={msg.content} />;
        }
        if (msg.role === "assistant") {
          return <AssistantMessage key={msg.id} content={msg.content} thinking={msg.thinking} />;
        }
        return null;
      })}
      {isStreaming && (
        <AssistantMessage content={streamingContent} thinking={streamingThinking} isStreaming />
      )}
      <div ref={bottomRef} />
    </div>
  );
}
