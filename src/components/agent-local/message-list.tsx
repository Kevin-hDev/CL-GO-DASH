import { memo } from "react";
import { UserMessage } from "./user-message";
import { AssistantMessage } from "./assistant-message";
import { SavedToolTimeline, StreamToolTimeline } from "./message-tool-timeline";
import { CompressionIndicator } from "./compression-indicator";
import { ContextCompressionMarker } from "./context-compression-marker";
import { SubagentBubble } from "./subagent-bubble";
import { PlanPreviewBubble } from "./plan-preview-bubble";
import { FileChangeBubble } from "./file-change-bubble";
import { LoadingIndicator } from "./working-stats";
import { useCompression } from "@/hooks/use-compression";
import { isCompressionContextOnlyMessage, isCompressionSummaryMessage } from "@/lib/context-messages";
import { collectMessageFileOperations } from "@/lib/file-preview-utils";
import { collectMessageSubagents } from "@/lib/message-subagents";
import type { AgentMessage, AgentPlanPreview, SubagentInfo, TokenPhase } from "@/types/agent";
import type { ToolActivity, StreamSegment } from "@/hooks/agent-chat-utils";
import type { ActiveStreamItem } from "@/hooks/active-stream-item";
import type { FileOperation } from "@/types/file-preview";
import "./chat.css";
import "./messages.css";

interface MessageListProps {
  sessionId: string;
  messages: AgentMessage[];
  completedSegments: StreamSegment[];
  currentContent: string;
  currentContentPhase?: TokenPhase;
  currentThinking: string;
  currentTools: ToolActivity[];
  activeStreamItem?: ActiveStreamItem;
  isStreaming: boolean;
  tps: number;
  totalElapsedMs: number;
  segmentStartedAt: number | null;
  liveTokenCount: number;
  onReload?: (messageId: string) => void;
  onEdit?: (messageId: string, newContent: string) => void;
  onCloneMessage?: (messageId: string) => void;
  onFileClick?: (file: { name: string; path?: string; thumbnail?: string }) => void;
  onFilePreview?: (path: string) => void;
  onFileReview?: (operation: FileOperation) => void;
  projectPath?: string;
  knownSubagents?: SubagentInfo[];
  onOpenSubagent?: (sessionId: string) => void;
  planPreview?: AgentPlanPreview | null;
}

export function MessageList({
  sessionId, messages, completedSegments, currentContent, currentContentPhase, currentThinking,
  currentTools, activeStreamItem = null, isStreaming, tps, totalElapsedMs, segmentStartedAt,
  liveTokenCount, onReload, onEdit, onCloneMessage, onFileClick, onFilePreview, onFileReview,
  projectPath, knownSubagents = [], onOpenSubagent, planPreview,
}: MessageListProps) {
  const lastAssistantIdx = findLastIndex(messages, (m) => m.role === "assistant");
  const { isCompressing } = useCompression(sessionId);
  const streamStartedAt = segmentStartedAt;
  const hasCompressionMarker = messages.some(isCompressionSummaryMessage);
  const showCompressionIndicator = isCompressing && !hasCompressionMarker;

  return (
    <>
      {messages.map((msg, idx) => {
        if (isCompressionSummaryMessage(msg)) return <ContextCompressionMarker key={msg.id} />;
        if (isCompressionContextOnlyMessage(msg)) return null;
        if (msg.role === "user") {
          return (
            <UserMessage
              key={msg.id} content={msg.content} files={msg.files}
              skillNames={msg.skill_names} isStreaming={isStreaming}
              onReload={onReload ? () => onReload(msg.id) : undefined}
              onEdit={onEdit ? (c) => onEdit(msg.id, c) : undefined}
              onClone={onCloneMessage ? () => onCloneMessage(msg.id) : undefined}
              onFileClick={onFileClick}
            />
          );
        }
        if (msg.role === "assistant") {
          const isLast = idx === lastAssistantIdx && !isStreaming;
          return (
            <SegmentedAssistantMessage
              key={msg.id} msg={msg} onReload={onReload}
              onClone={onCloneMessage}
              onFilePreview={onFilePreview}
              onFileReview={onFileReview}
              projectPath={projectPath}
              knownSubagents={knownSubagents}
              onOpenSubagent={onOpenSubagent}
              tps={isLast ? tps : 0}
              totalElapsedMs={isLast ? totalElapsedMs : 0}
              workDurationMs={msg.work_duration_ms}
            />
          );
        }
        return null;
      })}

      {isStreaming && (
        <StreamToolTimeline
          completedSegments={completedSegments}
          currentContent={currentContent}
          currentContentPhase={currentContentPhase}
          currentThinking={currentThinking}
          currentTools={currentTools}
          activeStreamItem={activeStreamItem}
          streamStartedAt={streamStartedAt}
          onFilePreview={onFilePreview}
          projectPath={projectPath}
        />
      )}
      {planPreview && <PlanPreviewBubble plan={planPreview} />}
      {isStreaming && streamStartedAt != null && (
        <LoadingIndicator startedAt={streamStartedAt} liveTokenCount={liveTokenCount} />
      )}

      {showCompressionIndicator && <CompressionIndicator />}

    </>
  );
}

export const SegmentedAssistantMessage = memo(function SegmentedAssistantMessage({
  msg, onReload, onClone, onFilePreview, onFileReview, tps, totalElapsedMs, workDurationMs,
  projectPath, knownSubagents = [], onOpenSubagent,
}: {
  msg: AgentMessage; onReload?: (id: string) => void; onFilePreview?: (path: string) => void;
  onClone?: (id: string) => void; onFileReview?: (operation: FileOperation) => void; tps: number; totalElapsedMs: number;
  workDurationMs?: number; projectPath?: string; knownSubagents?: SubagentInfo[];
  onOpenSubagent?: (sessionId: string) => void;
}) {
  const fileChanges = collectMessageFileOperations(msg, projectPath);
  const messageSubagents = collectMessageSubagents(msg, knownSubagents);
  if (msg.segments && msg.segments.length > 0) {
    return (
      <>
        <SavedToolTimeline
          messageId={msg.id}
          segments={msg.segments}
          tokens={msg.tokens}
          tps={tps}
          totalElapsedMs={workDurationMs ?? totalElapsedMs}
          onFilePreview={onFilePreview}
          projectPath={projectPath}
          onClone={() => onClone?.(msg.id)}
        />
        <SubagentBubble subagents={messageSubagents} onOpen={(id) => onOpenSubagent?.(id)} />
        <FileChangeBubble operations={fileChanges} baseDir={projectPath} onReview={onFileReview} />
      </>
    );
  }
  return (
    <>
      <AssistantMessage
        content={msg.content} thinking={msg.thinking}
        toolActivities={msg.tool_activities}
        projectPath={projectPath}
        onReload={onReload ? () => onReload(msg.id) : undefined}
        onClone={onClone ? () => onClone(msg.id) : undefined}
        tokens={msg.tokens}
        tps={tps}
      />
      <SubagentBubble subagents={messageSubagents} onOpen={(id) => onOpenSubagent?.(id)} />
      <FileChangeBubble operations={fileChanges} baseDir={projectPath} onReview={onFileReview} />
    </>
  );
});

function findLastIndex<T>(arr: T[], pred: (item: T) => boolean): number {
  for (let i = arr.length - 1; i >= 0; i--) {
    if (pred(arr[i])) return i;
  }
  return -1;
}
