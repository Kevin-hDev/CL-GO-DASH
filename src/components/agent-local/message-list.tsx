import { memo } from "react";
import { UserMessage } from "./user-message";
import { AssistantMessage } from "./assistant-message";
import { SavedToolTimeline, StreamToolTimeline } from "./message-tool-timeline";
import { CompressionIndicator } from "./compression-indicator";
import { ContextCompressionMarker } from "./context-compression-marker";
import { SubagentBubble } from "./subagent-bubble";
import { PlanPreviewBubble } from "./plan-preview-bubble";
import { LoadingIndicator } from "./working-stats";
import { useCompression } from "@/hooks/use-compression";
import { isCompressionContextOnlyMessage, isCompressionSummaryMessage } from "@/lib/context-messages";
import { isSubagentInjectedMessage, extractSubagentsFromMessages } from "@/lib/subagent-message-utils";
import type { AgentMessage, AgentPlanPreview, SubagentInfo, TokenPhase } from "@/types/agent";
import type { ToolActivity, StreamSegment } from "@/hooks/agent-chat-utils";
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
  isStreaming: boolean;
  tps: number;
  totalElapsedMs: number;
  segmentStartedAt: number | null;
  liveTokenCount: number;
  onReload?: (messageId: string) => void;
  onEdit?: (messageId: string, newContent: string) => void;
  onFileClick?: (file: { name: string; path?: string; thumbnail?: string }) => void;
  onFilePreview?: (path: string) => void;
  projectPath?: string;
  completedSubagents?: SubagentInfo[];
  onOpenSubagent?: (sessionId: string) => void;
  planPreview?: AgentPlanPreview | null;
}

export function MessageList({
  sessionId, messages, completedSegments, currentContent, currentContentPhase, currentThinking,
  currentTools, isStreaming, tps, totalElapsedMs, segmentStartedAt,
  liveTokenCount, onReload, onEdit, onFileClick, onFilePreview,
  projectPath, completedSubagents, onOpenSubagent, planPreview,
}: MessageListProps) {
  const lastAssistantIdx = findLastIndex(messages, (m) => m.role === "assistant");
  const { isCompressing } = useCompression(sessionId);
  const streamStartedAt = segmentStartedAt;

  const extractedAgents = completedSubagents && completedSubagents.length > 0
    ? completedSubagents
    : extractSubagentsFromMessages(messages);
  const subagentBubbleMessageId = extractedAgents.length > 0
    ? messages.find(isSubagentInjectedMessage)?.id
    : null;

  return (
    <>
      {messages.map((msg, idx) => {
        if (isCompressionSummaryMessage(msg)) return <ContextCompressionMarker key={msg.id} />;
        if (isCompressionContextOnlyMessage(msg)) return null;
        if (isSubagentInjectedMessage(msg)) {
          if (msg.id === subagentBubbleMessageId) {
            return (
              <SubagentBubble
                key={`sa-bubble-${msg.id}`}
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
              key={msg.id} content={msg.content} files={msg.files}
              skillNames={msg.skill_names} isStreaming={isStreaming}
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
              onFilePreview={onFilePreview}
              projectPath={projectPath}
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
          streamStartedAt={streamStartedAt}
          liveTokenCount={liveTokenCount}
          onFilePreview={onFilePreview}
          projectPath={projectPath}
        />
      )}
      {planPreview && <PlanPreviewBubble plan={planPreview} />}
      {isStreaming && streamStartedAt != null && !isCompressing && !currentContent && !currentThinking && !hasActiveTools(currentTools) && (
        <LoadingIndicator startedAt={streamStartedAt} liveTokenCount={liveTokenCount} />
      )}

      {isCompressing && <CompressionIndicator />}

    </>
  );
}

export const SegmentedAssistantMessage = memo(function SegmentedAssistantMessage({
  msg, onReload, onFilePreview, tps, totalElapsedMs, workDurationMs,
  projectPath,
}: {
  msg: AgentMessage; onReload?: (id: string) => void; onFilePreview?: (path: string) => void;
  tps: number; totalElapsedMs: number; workDurationMs?: number; projectPath?: string;
}) {
  if (msg.segments && msg.segments.length > 0) {
    return (
      <SavedToolTimeline
        messageId={msg.id}
        segments={msg.segments}
        tokens={msg.tokens}
        tps={tps}
        totalElapsedMs={workDurationMs ?? totalElapsedMs}
        onFilePreview={onFilePreview}
        projectPath={projectPath}
      />
    );
  }
  return (
    <AssistantMessage
      content={msg.content} thinking={msg.thinking}
      toolActivities={msg.tool_activities}
      projectPath={projectPath}
      onReload={onReload ? () => onReload(msg.id) : undefined}
      tokens={msg.tokens}
      tps={tps}
    />
  );
});

export function hasActiveTools(tools: ToolActivity[]): boolean {
  return tools.length > 0 && tools.some((t) => !t.result);
}

function findLastIndex<T>(arr: T[], pred: (item: T) => boolean): number {
  for (let i = arr.length - 1; i >= 0; i--) {
    if (pred(arr[i])) return i;
  }
  return -1;
}
