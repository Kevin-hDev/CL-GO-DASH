import { useRef, memo } from "react";
import { UserMessage } from "./user-message";
import { AssistantMessage } from "./assistant-message";
import { ToolBubble, SavedToolBubble } from "./tool-bubble";
import { ThinkingSection } from "./thinking-section";
import { ErrorBubble } from "./error-bubble";
import { CompressionIndicator } from "./compression-indicator";
import { ContextCompressionMarker } from "./context-compression-marker";
import { SubagentBubble } from "./subagent-bubble";
import { BranchBubble } from "./branch-bubble";
import { LoadingIndicator } from "./working-stats";
import { useCompression } from "@/hooks/use-compression";
import { isCompressionContextOnlyMessage, isCompressionSummaryMessage } from "@/lib/context-messages";
import { isSubagentInjectedMessage, extractSubagentsFromMessages } from "@/lib/subagent-message-utils";
import type { AgentMessage, SubagentInfo, ToolActivityRecord } from "@/types/agent";
import type { ToolActivity, StreamSegment } from "@/hooks/agent-chat-utils";
import "./chat.css";
import "./messages.css";

interface MessageListProps {
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
}

export function MessageList({
  sessionId, messages, completedSegments, currentContent, currentThinking,
  currentTools, isStreaming, tps, totalElapsedMs, segmentStartedAt,
  liveTokenCount, error, isConnectionError, onRetry, onReload, onEdit, onFileClick, onFilePreview,
  completedSubagents, onOpenSubagent,
}: MessageListProps) {
  const lastAssistantIdx = findLastIndex(messages, (m) => m.role === "assistant");
  const { isCompressing } = useCompression(sessionId);
  const fallbackRef = useRef<number | null>(null);
  if (isStreaming && !segmentStartedAt && fallbackRef.current === null) fallbackRef.current = Date.now();
  if (!isStreaming) fallbackRef.current = null;
  const streamStartedAt = segmentStartedAt ?? fallbackRef.current;
  const loadingStartedAt = streamStartedAt ?? Date.now();

  const extractedAgents = completedSubagents && completedSubagents.length > 0
    ? completedSubagents
    : extractSubagentsFromMessages(messages);
  let bubbleRendered = false;

  return (
    <>
      {messages.map((msg, idx) => {
        if (isCompressionSummaryMessage(msg)) return <ContextCompressionMarker key={msg.id} />;
        if (isCompressionContextOnlyMessage(msg)) return null;
        if (isSubagentInjectedMessage(msg)) {
          if (!bubbleRendered && extractedAgents.length > 0) {
            bubbleRendered = true;
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
              tps={isLast ? tps : 0}
              totalElapsedMs={isLast ? totalElapsedMs : 0}
            />
          );
        }
        return null;
      })}

      {isStreaming && completedSegments.map((seg, i) => (
        <div key={`seg-${i}`}>
          {seg.thinking && <ThinkingSection content={seg.thinking} />}
          {seg.content && <AssistantMessage content={seg.content} />}
          {seg.tools.length > 0 && <ToolBubble tools={seg.tools} onFilePreview={onFilePreview} />}
        </div>
      ))}

      {isStreaming && (currentContent || currentThinking) && (
        <AssistantMessage
          content={currentContent} thinking={currentThinking} isStreaming
          streamStartedAt={streamStartedAt} liveTokenCount={liveTokenCount}
        />
      )}
      {isStreaming && currentTools.length > 0 && <ToolBubble tools={currentTools} onFilePreview={onFilePreview} />}
      {isStreaming && !isCompressing && !currentContent && !currentThinking && !hasActiveTools(currentTools) && (
        <LoadingIndicator startedAt={loadingStartedAt} liveTokenCount={liveTokenCount} />
      )}

      {isCompressing && <CompressionIndicator />}

      {error && !isStreaming && (
        <ErrorBubble
          message={error}
          isConnection={isConnectionError}
          onRetry={onRetry}
        />
      )}
    </>
  );
}

export const SegmentedAssistantMessage = memo(function SegmentedAssistantMessage({
  msg, onReload, onFilePreview, tps, totalElapsedMs,
}: { msg: AgentMessage; onReload?: (id: string) => void; onFilePreview?: (path: string) => void; tps: number; totalElapsedMs: number }) {
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
                totalElapsedMs={i === lastSegIdx ? totalElapsedMs : undefined}
              />
            )}
            {seg.tools.length > 0 && <SavedToolBubble tools={seg.tools} onFilePreview={onFilePreview} />}
            {(() => { const b = extractBranchActivity(seg.tools); return b ? <BranchBubble action={b.action} branchName={b.branchName} path={b.path} /> : null; })()}
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
});

export function hasActiveTools(tools: ToolActivity[]): boolean {
  return tools.length > 0 && tools.some((t) => !t.result);
}

function extractBranchActivity(tools: ToolActivityRecord[]): { action: "created" | "switched"; branchName: string; path?: string } | null {
  for (const t of tools) {
    if (t.name === "create_branch" && t.result && !t.is_error) {
      return { action: "created", branchName: t.summary, path: t.result };
    }
    if (t.name === "checkout_branch" && t.result && !t.is_error) {
      return { action: "switched", branchName: t.summary };
    }
  }
  return null;
}

function findLastIndex<T>(arr: T[], pred: (item: T) => boolean): number {
  for (let i = arr.length - 1; i >= 0; i--) {
    if (pred(arr[i])) return i;
  }
  return -1;
}
