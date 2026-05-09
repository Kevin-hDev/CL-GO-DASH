import { memo } from "react";
import { useTranslation } from "react-i18next";
import { CodeBlock } from "./code-block";
import { ThinkingSection } from "./thinking-section";
import { MessageActions } from "./message-actions";
import { SavedToolBubble } from "./tool-bubble";
import { StreamingStats, formatTotalElapsed } from "./streaming-stats";
import { useHoverClass } from "@/hooks/use-hover-class";
import { linkify } from "@/lib/linkify";
import type { ToolActivityRecord } from "@/types/agent";
import "./messages.css";

interface AssistantMessageProps {
  content: string;
  thinking?: string;
  toolActivities?: ToolActivityRecord[];
  isStreaming?: boolean;
  onReload?: () => void;
  tokens?: number;
  tps?: number;
  totalElapsedMs?: number;
  streamStartedAt?: number | null;
  liveTokenCount?: number;
}

function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

export const AssistantMessage = memo(function AssistantMessage({
  content, thinking, toolActivities, isStreaming, onReload,
  tokens, tps, totalElapsedMs, streamStartedAt, liveTokenCount,
}: AssistantMessageProps) {
  const { t } = useTranslation();
  const hoverRef = useHoverClass();
  const hasTokens = tokens != null && tokens > 0;
  const hasTps = tps != null && tps > 0.1;
  const totalTime = formatTotalElapsed(totalElapsedMs ?? 0);

  return (
    <div className="msg-assistant" ref={hoverRef}>
      {thinking && <ThinkingSection content={thinking} isActive={isStreaming && !content} />}
      {toolActivities && toolActivities.length > 0 && (
        <SavedToolBubble tools={toolActivities} />
      )}
      <div className="msg-assistant-content">
        {content && renderMarkdown(content)}
        {isStreaming && (
          <>
            <span style={{ animation: "pulse-dot 1s infinite" }}>▊</span>
            {!content && (
              <StreamingStats segmentStartedAt={streamStartedAt ?? null} liveTokenCount={liveTokenCount ?? 0} />
            )}
          </>
        )}
      </div>
      {!isStreaming && content.trim() && (
        <MessageActions messageRole="assistant" content={content} onReload={onReload}>
          {(hasTokens || hasTps || totalTime) && (
            <span className="msg-stats-inline">
              {totalTime && <><span>{totalTime}</span><span>·</span></>}
              {hasTokens && <span>{formatTokens(tokens)} {t("agentLocal.tokens")}</span>}
              {hasTokens && hasTps && <span>·</span>}
              {hasTps && <span>{tps.toFixed(1)} {t("agentLocal.tps")}</span>}
            </span>
          )}
        </MessageActions>
      )}
    </div>
  );
});

function renderMarkdown(text: string) {
  const parts: React.ReactNode[] = [];
  const codeBlockRegex = /```(\w*)\n([\s\S]*?)```/g;
  let lastIndex = 0;
  let match;

  while ((match = codeBlockRegex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push(<span key={lastIndex}>{linkify(text.slice(lastIndex, match.index))}</span>);
    }
    parts.push(<CodeBlock key={match.index} language={match[1]} code={match[2]} />);
    lastIndex = match.index + match[0].length;
  }

  if (lastIndex < text.length) {
    const remaining = text.slice(lastIndex);
    const openBlock = /```(\w*)\n([\s\S]*)$/.exec(remaining);
    if (openBlock) {
      const before = remaining.slice(0, openBlock.index);
      if (before) parts.push(<span key={lastIndex}>{linkify(before)}</span>);
      parts.push(<CodeBlock key={lastIndex + openBlock.index} language={openBlock[1]} code={openBlock[2]} />);
    } else {
      parts.push(<span key={lastIndex}>{linkify(remaining)}</span>);
    }
  }

  return parts.length > 0 ? parts : text;
}
