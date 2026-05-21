import { AssistantMessage } from "./assistant-message";
import { BranchBubble } from "./branch-bubble";
import { ThinkingSection } from "./thinking-section";
import { ToolBubble, SavedToolBubble } from "./tool-bubble";
import { buildToolTimelineBlocks, hasNarrative } from "./message-tool-blocks";
import { extractBranchActivity } from "./message-tool-aggregation";
import type { ToolActivity, StreamSegment } from "@/hooks/agent-chat-utils";
import type { SavedSegment } from "@/types/agent";

interface StreamToolTimelineProps {
  completedSegments: StreamSegment[];
  currentContent: string;
  currentThinking: string;
  currentTools: ToolActivity[];
  streamStartedAt: number | null;
  liveTokenCount: number;
  onFilePreview?: (path: string) => void;
}

export function StreamToolTimeline({
  completedSegments,
  currentContent,
  currentThinking,
  currentTools,
  streamStartedAt,
  liveTokenCount,
  onFilePreview,
}: StreamToolTimelineProps) {
  const segments = [
    ...completedSegments,
    ...(currentContent || currentThinking || currentTools.length > 0
      ? [{ thinking: currentThinking, content: currentContent, tools: currentTools, isCurrent: true }]
      : []),
  ];
  const blocks = buildToolTimelineBlocks(segments);
  return (
    <>
      {blocks.map((block, index) => (
        <div key={`stream-block-${index}`}>
          {block.isCurrent && hasNarrative(block) ? (
            <AssistantMessage
              content={block.content ?? ""}
              thinking={block.thinking}
              isStreaming
              streamStartedAt={streamStartedAt}
              liveTokenCount={liveTokenCount}
            />
          ) : (
            <>
              {block.thinking && <ThinkingSection content={block.thinking} />}
              {block.content && <AssistantMessage content={block.content} />}
            </>
          )}
          {block.tools.length > 0 && <ToolBubble tools={block.tools} onFilePreview={onFilePreview} />}
        </div>
      ))}
    </>
  );
}

interface SavedToolTimelineProps {
  messageId: string;
  segments: SavedSegment[];
  tokens?: number;
  tps: number;
  totalElapsedMs: number;
  onFilePreview?: (path: string) => void;
}

export function SavedToolTimeline({
  messageId,
  segments,
  tokens,
  tps,
  totalElapsedMs,
  onFilePreview,
}: SavedToolTimelineProps) {
  const blocks = buildToolTimelineBlocks(segments);
  const lastTextIndex = findLastIndex(blocks, (block) => !!block.content);
  return (
    <>
      {blocks.map((block, index) => {
        const branchActivity = extractBranchActivity(block.tools);
        const showStats = index === lastTextIndex;
        return (
          <div key={`${messageId}-block-${index}`}>
            {block.thinking && <ThinkingSection content={block.thinking} />}
            {block.content && (
              <AssistantMessage
                content={block.content}
                tokens={showStats ? tokens : undefined}
                tps={showStats ? tps : undefined}
                totalElapsedMs={showStats ? totalElapsedMs : undefined}
              />
            )}
            {block.tools.length > 0 && <SavedToolBubble tools={block.tools} onFilePreview={onFilePreview} />}
            {branchActivity && (
              <BranchBubble
                action={branchActivity.action}
                branchName={branchActivity.branchName}
                path={branchActivity.path}
              />
            )}
          </div>
        );
      })}
    </>
  );
}

function findLastIndex<T>(arr: T[], pred: (item: T) => boolean): number {
  for (let i = arr.length - 1; i >= 0; i--) {
    if (pred(arr[i])) return i;
  }
  return -1;
}
