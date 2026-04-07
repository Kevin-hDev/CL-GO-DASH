import { useLottie } from "lottie-react";
import { UserMessage } from "./user-message";
import { AssistantMessage } from "./assistant-message";
import type { AgentMessage } from "@/types/agent";
import thinkingAnimation from "@/assets/thinking-loader.json";
import "./chat.css";

interface MessageListProps {
  messages: AgentMessage[];
  streamingContent: string;
  streamingThinking: string;
  isStreaming: boolean;
  onReload?: (messageId: string) => void;
  onEdit?: (messageId: string, newContent: string) => void;
}

export function MessageList({
  messages, streamingContent, streamingThinking, isStreaming,
  onReload, onEdit,
}: MessageListProps) {
  return (
    <>
      {messages.map((msg) => {
        if (msg.role === "user") {
          return (
            <UserMessage
              key={msg.id}
              content={msg.content}
              files={msg.files}
              onReload={onReload ? () => onReload(msg.id) : undefined}
              onEdit={onEdit ? (c) => onEdit(msg.id, c) : undefined}
            />
          );
        }
        if (msg.role === "assistant") {
          return (
            <AssistantMessage
              key={msg.id}
              content={msg.content}
              thinking={msg.thinking}
              onReload={onReload ? () => onReload(msg.id) : undefined}
            />
          );
        }
        return null;
      })}
      {isStreaming && streamingContent.length < 1 && <LoadingIndicator />}
      {isStreaming && streamingContent.length > 0 && (
        <AssistantMessage content={streamingContent} thinking={streamingThinking} isStreaming />
      )}
    </>
  );
}

function LoadingIndicator() {
  const { View } = useLottie({
    animationData: thinkingAnimation,
    loop: true,
    className: "chat-loading-lottie",
  });

  return <div className="chat-loading">{View}</div>;
}
