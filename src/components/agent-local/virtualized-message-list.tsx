import { useRef, useCallback, useMemo, memo } from "react";
import { Virtuoso, type VirtuosoHandle } from "react-virtuoso";
import { UserMessage } from "./user-message";
import { SegmentedAssistantMessage } from "./message-list";
import { SubagentBubble } from "./subagent-bubble";
import { StreamingFooter } from "./streaming-footer";
import { isSubagentInjectedMessage, extractSubagentsFromMessages } from "@/lib/subagent-message-utils";
import type { AgentMessage, SubagentInfo } from "@/types/agent";
import type { ToolActivity, StreamSegment } from "@/hooks/agent-chat-utils";
import "./chat.css";
import "./messages.css";

interface VirtualizedMessageListProps {
  sessionId: string;
  messages: AgentMessage[];
  completedSegments: StreamSegment[];
  currentContent: string;
  currentThinking: string;
  currentTools: ToolActivity[];
  isStreaming: boolean;
  tps: number;
  totalElapsedMs: number;
  segmentStartedAt: number | null;
  liveTokenCount: number;
  error?: string;
  isConnectionError?: boolean;
  onRetry?: () => void;
  onReload?: (messageId: string) => void;
  onEdit?: (messageId: string, newContent: string) => void;
  onFileClick?: (file: { name: string; path?: string; thumbnail?: string }) => void;
  onFilePreview?: (path: string) => void;
  completedSubagents?: SubagentInfo[];
  onOpenSubagent?: (sessionId: string) => void;
  isAtBottom: boolean;
  onAtBottomChange: (atBottom: boolean) => void;
}

function findLastIndex<T>(arr: T[], pred: (item: T) => boolean): number {
  for (let i = arr.length - 1; i >= 0; i--) {
    if (pred(arr[i])) return i;
  }
  return -1;
}

const MessageItemRenderer = memo(function MessageItemRenderer({
  msg, isLastAssistant, isStreaming, tps, totalElapsedMs,
  extractedAgents, isFirstSubagentMsg,
  onReload, onEdit, onFileClick, onFilePreview, onOpenSubagent,
}: {
  msg: AgentMessage;
  isLastAssistant: boolean;
  isStreaming: boolean;
  tps: number;
  totalElapsedMs: number;
  extractedAgents: SubagentInfo[];
  isFirstSubagentMsg: boolean;
  onReload?: (messageId: string) => void;
  onEdit?: (messageId: string, newContent: string) => void;
  onFileClick?: (file: { name: string; path?: string; thumbnail?: string }) => void;
  onFilePreview?: (path: string) => void;
  onOpenSubagent?: (sessionId: string) => void;
}) {
  if (isSubagentInjectedMessage(msg)) {
    if (isFirstSubagentMsg && extractedAgents.length > 0) {
      return (
        <SubagentBubble
          subagents={extractedAgents}
          onOpen={(id) => onOpenSubagent?.(id)}
        />
      );
    }
    return null;
  }
  if (msg.role === "user") {
    return (
      <UserMessage
        content={msg.content} files={msg.files}
        skillNames={msg.skill_names} isStreaming={isStreaming}
        onReload={onReload ? () => onReload(msg.id) : undefined}
        onEdit={onEdit ? (c) => onEdit(msg.id, c) : undefined}
        onFileClick={onFileClick}
      />
    );
  }
  if (msg.role === "assistant") {
    return (
      <SegmentedAssistantMessage
        msg={msg} onReload={onReload} onFilePreview={onFilePreview}
        tps={isLastAssistant && !isStreaming ? tps : 0}
        totalElapsedMs={isLastAssistant && !isStreaming ? totalElapsedMs : 0}
      />
    );
  }
  return null;
});

export function VirtualizedMessageList({
  sessionId, messages, completedSegments, currentContent, currentThinking,
  currentTools, isStreaming, tps, totalElapsedMs, segmentStartedAt,
  liveTokenCount, error, isConnectionError, onRetry, onReload, onEdit,
  onFileClick, onFilePreview, completedSubagents, onOpenSubagent,
  onAtBottomChange,
}: VirtualizedMessageListProps) {
  const virtuosoRef = useRef<VirtuosoHandle>(null);

  const extractedAgents = completedSubagents && completedSubagents.length > 0
    ? completedSubagents
    : extractSubagentsFromMessages(messages);

  const lastAssistantIdx = findLastIndex(messages, (m) => m.role === "assistant");

  const firstSubagentIdx = useMemo(() => {
    return messages.findIndex((m) => isSubagentInjectedMessage(m));
  }, [messages]);

  const renderItem = useCallback((index: number, msg: AgentMessage) => {
    return (
      <MessageItemRenderer
        msg={msg}
        isLastAssistant={index === lastAssistantIdx}
        isStreaming={isStreaming}
        tps={tps}
        totalElapsedMs={totalElapsedMs}
        extractedAgents={extractedAgents}
        isFirstSubagentMsg={index === firstSubagentIdx}
        onReload={onReload}
        onEdit={onEdit}
        onFileClick={onFileClick}
        onFilePreview={onFilePreview}
        onOpenSubagent={onOpenSubagent}
      />
    );
  }, [lastAssistantIdx, isStreaming, tps, totalElapsedMs, extractedAgents, firstSubagentIdx, onReload, onEdit, onFileClick, onFilePreview, onOpenSubagent]);

  const footer = useCallback(() => (
    <StreamingFooter
      sessionId={sessionId}
      completedSegments={completedSegments}
      currentContent={currentContent}
      currentThinking={currentThinking}
      currentTools={currentTools}
      isStreaming={isStreaming}
      segmentStartedAt={segmentStartedAt}
      liveTokenCount={liveTokenCount}
      error={error}
      isConnectionError={isConnectionError}
      onRetry={onRetry}
      onFilePreview={onFilePreview}
    />
  ), [sessionId, completedSegments, currentContent, currentThinking, currentTools, isStreaming, segmentStartedAt, liveTokenCount, error, isConnectionError, onRetry, onFilePreview]);

  const followOutput = useCallback((atBottom: boolean) => {
    return atBottom ? "smooth" as const : false as const;
  }, []);

  return (
    <Virtuoso
      ref={virtuosoRef}
      data={messages}
      itemContent={renderItem}
      components={{ Footer: footer }}
      followOutput={followOutput}
      atBottomStateChange={onAtBottomChange}
      atBottomThreshold={80}
      overscan={300}
      initialTopMostItemIndex={messages.length > 0 ? messages.length - 1 : 0}
      style={{ height: "100%", width: "100%" }}
    />
  );
}
