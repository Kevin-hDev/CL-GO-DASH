import { useRef, useCallback, useMemo, useEffect, memo } from "react";
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
  streamStartedAt: number | null;
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
  scrollActionRef?: React.MutableRefObject<(() => void) | null>;
}

interface RenderableItem {
  type: "user" | "assistant" | "subagent-bubble";
  msg: AgentMessage;
  isLastAssistant: boolean;
}

interface VolatileState {
  isStreaming: boolean;
  tps: number;
  totalElapsedMs: number;
  extractedAgents: SubagentInfo[];
}

const RenderableItemView = memo(function RenderableItemView({
  item, volatileRef,
  onReload, onEdit, onFileClick, onFilePreview, onOpenSubagent,
}: {
  item: RenderableItem;
  volatileRef: React.RefObject<VolatileState>;
  onReload?: (messageId: string) => void;
  onEdit?: (messageId: string, newContent: string) => void;
  onFileClick?: (file: { name: string; path?: string; thumbnail?: string }) => void;
  onFilePreview?: (path: string) => void;
  onOpenSubagent?: (sessionId: string) => void;
}) {
  const { isStreaming, tps, totalElapsedMs, extractedAgents } = volatileRef.current;

  if (item.type === "subagent-bubble") {
    return (
      <SubagentBubble
        subagents={extractedAgents}
        onOpen={(id) => onOpenSubagent?.(id)}
      />
    );
  }
  if (item.type === "user") {
    return (
      <UserMessage
        content={item.msg.content} files={item.msg.files}
        skillNames={item.msg.skill_names} isStreaming={isStreaming}
        onReload={onReload ? () => onReload(item.msg.id) : undefined}
        onEdit={onEdit ? (c) => onEdit(item.msg.id, c) : undefined}
        onFileClick={onFileClick}
      />
    );
  }
  return (
    <SegmentedAssistantMessage
      msg={item.msg} onReload={onReload} onFilePreview={onFilePreview}
      tps={item.isLastAssistant && !isStreaming ? tps : 0}
      totalElapsedMs={item.isLastAssistant && !isStreaming ? totalElapsedMs : 0}
    />
  );
});

export function VirtualizedMessageList({
  sessionId, messages, completedSegments, currentContent, currentThinking,
  currentTools, isStreaming, tps, totalElapsedMs, streamStartedAt,
  liveTokenCount, error, isConnectionError, onRetry, onReload, onEdit,
  onFileClick, onFilePreview, completedSubagents, onOpenSubagent,
  onAtBottomChange, scrollActionRef,
}: VirtualizedMessageListProps) {
  const virtuosoRef = useRef<VirtuosoHandle>(null);

  const extractedAgents = useMemo(() =>
    completedSubagents && completedSubagents.length > 0
      ? completedSubagents
      : extractSubagentsFromMessages(messages),
    [completedSubagents, messages],
  );

  const volatileRef = useRef<VolatileState>({ isStreaming: false, tps: 0, totalElapsedMs: 0, extractedAgents: [] });
  volatileRef.current = { isStreaming, tps, totalElapsedMs, extractedAgents };

  const renderableItems = useMemo(() => {
    const items: RenderableItem[] = [];
    let subagentBubblePlaced = false;
    let lastAssistantIdx = -1;
    for (let i = messages.length - 1; i >= 0; i--) {
      if (messages[i].role === "assistant") { lastAssistantIdx = i; break; }
    }
    for (let i = 0; i < messages.length; i++) {
      const msg = messages[i];
      if (isSubagentInjectedMessage(msg)) {
        if (!subagentBubblePlaced && extractedAgents.length > 0) {
          subagentBubblePlaced = true;
          items.push({ type: "subagent-bubble", msg, isLastAssistant: false });
        }
        continue;
      }
      if (msg.role === "user") {
        items.push({ type: "user", msg, isLastAssistant: false });
      } else if (msg.role === "assistant") {
        items.push({ type: "assistant", msg, isLastAssistant: i === lastAssistantIdx });
      }
    }
    return items;
  }, [messages, extractedAgents]);

  if (scrollActionRef) {
    scrollActionRef.current = () => {
      virtuosoRef.current?.scrollToIndex({
        index: renderableItems.length - 1,
        align: "end",
        behavior: "smooth",
      });
    };
  }

  const atBottomRef = useRef(true);
  const handleAtBottomChange = useCallback((atBottom: boolean) => {
    atBottomRef.current = atBottom;
    onAtBottomChange(atBottom);
  }, [onAtBottomChange]);

  useEffect(() => {
    if (!isStreaming || !atBottomRef.current) return;
    const raf = requestAnimationFrame(() => {
      virtuosoRef.current?.scrollTo({ top: Number.MAX_SAFE_INTEGER });
    });
    return () => cancelAnimationFrame(raf);
  }, [currentContent, currentThinking, currentTools, completedSegments, isStreaming]);

  const renderItem = useCallback((_index: number, item: RenderableItem) => {
    return (
      <div className="chat-msg-item">
        <RenderableItemView
          item={item}
          volatileRef={volatileRef}
          onReload={onReload}
          onEdit={onEdit}
          onFileClick={onFileClick}
          onFilePreview={onFilePreview}
          onOpenSubagent={onOpenSubagent}
        />
      </div>
    );
  }, [onReload, onEdit, onFileClick, onFilePreview, onOpenSubagent]);

  const footer = useCallback(() => (
    <StreamingFooter
      sessionId={sessionId}
      completedSegments={completedSegments}
      currentContent={currentContent}
      currentThinking={currentThinking}
      currentTools={currentTools}
      isStreaming={isStreaming}
      streamStartedAt={streamStartedAt}
      liveTokenCount={liveTokenCount}
      error={error}
      isConnectionError={isConnectionError}
      onRetry={onRetry}
      onFilePreview={onFilePreview}
    />
  ), [sessionId, completedSegments, currentContent, currentThinking, currentTools, isStreaming, streamStartedAt, liveTokenCount, error, isConnectionError, onRetry, onFilePreview]);

  const virtuosoComponents = useMemo(() => ({ Footer: footer }), [footer]);

  const computeItemKey = useCallback(
    (_index: number, item: RenderableItem) => `${item.type}:${item.msg.id}`,
    [],
  );

  return (
    <Virtuoso
      key={sessionId}
      ref={virtuosoRef}
      data={renderableItems}
      computeItemKey={computeItemKey}
      itemContent={renderItem}
      components={virtuosoComponents}
      followOutput={(atBottom: boolean) => atBottom ? "smooth" as const : false as const}
      atBottomStateChange={handleAtBottomChange}
      atBottomThreshold={80}
      increaseViewportBy={{ top: 600, bottom: 800 }}
      initialTopMostItemIndex={renderableItems.length > 0 ? renderableItems.length - 1 : 0}
      style={{ height: "100%", width: "100%" }}
    />
  );
}
