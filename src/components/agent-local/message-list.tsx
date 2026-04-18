import { useLottie } from "lottie-react";
import { UserMessage } from "./user-message";
import { AssistantMessage } from "./assistant-message";
import { ToolBubble, SavedToolBubble } from "./tool-bubble";
import { ThinkingSection } from "./thinking-section";
import type { AgentMessage } from "@/types/agent";
import type { ToolActivity, StreamSegment } from "@/hooks/agent-chat-utils";
import thinkingAnimation from "@/assets/thinking-loader.json";
import "./chat.css";

interface MessageListProps {
  messages: AgentMessage[];
  completedSegments: StreamSegment[];
  currentContent: string;
  currentThinking: string;
  currentTools: ToolActivity[];
  isStreaming: boolean;
  tps: number;
  onReload?: (messageId: string) => void;
  onEdit?: (messageId: string, newContent: string) => void;
  onFileClick?: (file: { name: string; path?: string; thumbnail?: string }) => void;
}

export function MessageList({
  messages, completedSegments, currentContent, currentThinking,
  currentTools, isStreaming, tps, onReload, onEdit, onFileClick,
}: MessageListProps) {
  const lastAssistantIdx = findLastIndex(messages, (m) => m.role === "assistant");

  return (
    <>
      {messages.map((msg, idx) => {
        if (msg.role === "user") {
          return (
            <UserMessage
              key={msg.id} content={msg.content} files={msg.files}
              onReload={onReload ? () => onReload(msg.id) : undefined}
              onEdit={onEdit ? (c) => onEdit(msg.id, c) : undefined}
              onFileClick={onFileClick}
            />
          );
        }
        if (msg.role === "assistant") {
          const isLast = idx === lastAssistantIdx && !isStreaming;
          return (
            <SegmentedAssistantMessage
              key={msg.id} msg={msg} onReload={onReload}
              tps={isLast ? tps : 0}
            />
          );
        }
        return null;
      })}

      {isStreaming && completedSegments.map((seg, i) => (
        <div key={`seg-${i}`}>
          {seg.thinking && <ThinkingSection content={seg.thinking} />}
          {seg.content && <AssistantMessage content={seg.content} />}
          {seg.tools.length > 0 && <ToolBubble tools={seg.tools} />}
        </div>
      ))}

      {isStreaming && !currentContent && currentTools.length < 1 && completedSegments.length < 1 && (
        <LoadingIndicator />
      )}
      {isStreaming && (currentContent || currentThinking) && (
        <AssistantMessage content={currentContent} thinking={currentThinking} isStreaming />
      )}
      {isStreaming && currentTools.length > 0 && <ToolBubble tools={currentTools} />}
    </>
  );
}

function SegmentedAssistantMessage({
  msg, onReload, tps,
}: { msg: AgentMessage; onReload?: (id: string) => void; tps: number }) {
  if (msg.segments && msg.segments.length > 0) {
    const lastSegIdx = msg.segments.length - 1;
    return (
      <>
        {msg.segments.map((seg, i) => (
          <div key={`${msg.id}-seg-${i}`}>
            {seg.thinking && <ThinkingSection content={seg.thinking} />}
            {seg.content && (
              <AssistantMessage
                content={seg.content}
                tokens={i === lastSegIdx ? msg.tokens : undefined}
                tps={i === lastSegIdx ? tps : undefined}
              />
            )}
            {seg.tools.length > 0 && <SavedToolBubble tools={seg.tools} />}
          </div>
        ))}
      </>
    );
  }
  return (
    <AssistantMessage
      content={msg.content} thinking={msg.thinking}
      toolActivities={msg.tool_activities}
      onReload={onReload ? () => onReload(msg.id) : undefined}
      tokens={msg.tokens}
      tps={tps}
    />
  );
}

function findLastIndex<T>(arr: T[], pred: (item: T) => boolean): number {
  for (let i = arr.length - 1; i >= 0; i--) {
    if (pred(arr[i])) return i;
  }
  return -1;
}

function LoadingIndicator() {
  const { View } = useLottie({
    animationData: thinkingAnimation, loop: true, className: "chat-loading-lottie",
  });
  return <div className="chat-loading">{View}</div>;
}
