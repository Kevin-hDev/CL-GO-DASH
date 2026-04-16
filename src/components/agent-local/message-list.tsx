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
  onReload?: (messageId: string) => void;
  onEdit?: (messageId: string, newContent: string) => void;
  onFileClick?: (file: { name: string; path?: string; thumbnail?: string }) => void;
}

export function MessageList({
  messages, completedSegments, currentContent, currentThinking,
  currentTools, isStreaming, onReload, onEdit, onFileClick,
}: MessageListProps) {
  return (
    <>
      {messages.map((msg) => {
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
          return <SegmentedAssistantMessage key={msg.id} msg={msg} onReload={onReload} />;
        }
        return null;
      })}

      {/* Segments terminés (tours précédents) — ordre : thinking → content → tools */}
      {isStreaming && completedSegments.map((seg, i) => (
        <div key={`seg-${i}`}>
          {seg.thinking && <ThinkingSection content={seg.thinking} />}
          {seg.content && <AssistantMessage content={seg.content} />}
          {seg.tools.length > 0 && <ToolBubble tools={seg.tools} />}
        </div>
      ))}

      {/* Segment en cours — même ordre */}
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
  msg, onReload,
}: { msg: AgentMessage; onReload?: (id: string) => void }) {
  const tokensFooter = msg.tokens && msg.tokens > 0 ? (
    <div
      style={{
        fontSize: "11px",
        color: "var(--ink-faint)",
        fontFamily: "var(--font-mono, monospace)",
        padding: "0 var(--space-md)",
        marginTop: -4,
        marginBottom: "var(--space-sm)",
        opacity: 0.7,
      }}
      title="Tokens consommés par cet échange (input + output)"
    >
      {formatTokens(msg.tokens)} tokens
    </div>
  ) : null;

  if (msg.segments && msg.segments.length > 0) {
    return (
      <>
        {msg.segments.map((seg, i) => (
          <div key={`${msg.id}-seg-${i}`}>
            {seg.thinking && <ThinkingSection content={seg.thinking} />}
            {seg.content && (
              <AssistantMessage content={seg.content} />
            )}
            {seg.tools.length > 0 && <SavedToolBubble tools={seg.tools} />}
          </div>
        ))}
        {tokensFooter}
      </>
    );
  }
  return (
    <>
      <AssistantMessage
        content={msg.content} thinking={msg.thinking}
        toolActivities={msg.tool_activities}
        onReload={onReload ? () => onReload(msg.id) : undefined}
      />
      {tokensFooter}
    </>
  );
}

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

function LoadingIndicator() {
  const { View } = useLottie({
    animationData: thinkingAnimation, loop: true, className: "chat-loading-lottie",
  });
  return <div className="chat-loading">{View}</div>;
}
