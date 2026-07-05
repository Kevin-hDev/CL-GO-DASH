import { AssistantMessage } from "./assistant-message";
import { BranchBubble } from "./branch-bubble";
import { ThinkingSection } from "./thinking-section";
import { SavedToolBubble } from "./tool-bubble";
import { buildToolTimelineBlocks } from "./message-tool-blocks";
import { extractBranchActivity } from "./message-tool-aggregation";
import {
  hasWorkContent,
  isFinalStreamPhase,
  LiveWorkStreamSummary,
  savedWorkBlocks,
  TimelineLiveBlock,
  TimelineWorkBlock,
} from "./message-tool-timeline-render";
import { WorkStreamSummary } from "./work-stream-summary";
import type { ToolActivity, StreamSegment } from "@/hooks/agent-chat-utils";
import type { SavedSegment, TokenPhase } from "@/types/agent";

interface StreamToolTimelineProps {
  completedSegments: StreamSegment[];
  currentContent: string;
  currentContentPhase?: TokenPhase;
  currentThinking: string;
  currentTools: ToolActivity[];
  streamStartedAt: number | null;
  liveTokenCount: number;
  onFilePreview?: (path: string) => void;
  projectPath?: string;
}

export function StreamToolTimeline({
  completedSegments,
  currentContent,
  currentContentPhase,
  currentThinking,
  currentTools,
  streamStartedAt,
  liveTokenCount,
  onFilePreview,
  projectPath,
}: StreamToolTimelineProps) {
  const segments = [
    ...completedSegments,
    ...(currentContent || currentThinking || currentTools.length > 0
      ? [{
        thinking: currentThinking,
        content: currentContent,
        tools: currentTools,
        isCurrent: true,
        phase: currentContentPhase,
      }]
      : []),
  ];
  const blocks = buildToolTimelineBlocks(segments);
  const finalIndex = isFinalStreamPhase(blocks, currentContent, currentContentPhase);
  if (finalIndex >= 0) {
    const finalBlock = blocks[finalIndex];
    const workBlocks = savedWorkBlocks(blocks, finalIndex);
    return (
      <>
        <LiveWorkStreamSummary startedAt={streamStartedAt}>
          {workBlocks.map((block, index) => (
            <TimelineWorkBlock
              key={`stream-work-${index}`}
              block={block}
              bubbleKind="stream"
              onFilePreview={onFilePreview}
              projectPath={projectPath}
            />
          ))}
        </LiveWorkStreamSummary>
        <AssistantMessage
          content={finalBlock.content ?? ""}
          isStreaming
          streamStartedAt={streamStartedAt}
          liveTokenCount={liveTokenCount}
        />
      </>
    );
  }

  return (
    <>
      {blocks.map((block, index) => (
        <TimelineLiveBlock
          key={`stream-block-${index}`}
          block={block}
          streamStartedAt={streamStartedAt}
          liveTokenCount={liveTokenCount}
          onFilePreview={onFilePreview}
          projectPath={projectPath}
        />
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
  onClone?: () => void;
  projectPath?: string;
}

export function SavedToolTimeline({
  messageId,
  segments,
  tokens,
  tps,
  totalElapsedMs,
  onFilePreview,
  onClone,
  projectPath,
}: SavedToolTimelineProps) {
  const blocks = buildToolTimelineBlocks(segments);
  const hasPhase = blocks.some((block) => !!block.phase);
  const explicitFinalIndex = findLastIndex(blocks, (block) => block.phase === "final" && !!block.content);
  const lastTextIndex = explicitFinalIndex >= 0
    ? explicitFinalIndex
    : hasPhase ? -1 : findLastIndex(blocks, (block) => !!block.content);
  if (lastTextIndex >= 0) {
    const finalBlock = blocks[lastTextIndex];
    const workBlocks = savedWorkBlocks(blocks, lastTextIndex);
    const hasWork = workBlocks.some(hasWorkContent);
    if (hasWork) {
      return (
        <>
          <WorkStreamSummary durationMs={totalElapsedMs > 0 ? totalElapsedMs : undefined}>
            {workBlocks.map((block, index) => (
              <TimelineWorkBlock
                key={`${messageId}-work-${index}`}
                block={block}
                bubbleKind="saved"
                onFilePreview={onFilePreview}
                projectPath={projectPath}
              />
            ))}
          </WorkStreamSummary>
          <AssistantMessage
            content={finalBlock.content ?? ""}
            tokens={tokens}
            tps={tps}
            onClone={onClone}
          />
        </>
      );
    }
    return (
      <AssistantMessage
        content={finalBlock.content ?? ""}
        tokens={tokens}
        tps={tps}
        totalElapsedMs={totalElapsedMs}
        onClone={onClone}
      />
    );
  }
  if (hasPhase && blocks.some(hasWorkContent)) {
    return (
      <WorkStreamSummary durationMs={totalElapsedMs > 0 ? totalElapsedMs : undefined}>
        {blocks.map((block, index) => (
          <TimelineWorkBlock
            key={`${messageId}-work-only-${index}`}
            block={block}
            bubbleKind="saved"
            onFilePreview={onFilePreview}
            projectPath={projectPath}
          />
        ))}
      </WorkStreamSummary>
    );
  }

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
                onClone={showStats ? onClone : undefined}
              />
            )}
            {block.tools.length > 0 && (
              <SavedToolBubble tools={block.tools} onFilePreview={onFilePreview} projectPath={projectPath} />
            )}
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
