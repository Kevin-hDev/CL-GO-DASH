import { AssistantMessage } from "./assistant-message";
import { ToolBubble } from "./tool-bubble";
import { ThinkingSection } from "./thinking-section";
import { ErrorBubble } from "./error-bubble";
import { CompressionIndicator } from "./compression-indicator";
import { LoadingIndicator } from "./working-stats";
import { useCompression } from "@/hooks/use-compression";
import { hasActiveTools } from "./message-list";
import type { ToolActivity, StreamSegment } from "@/hooks/agent-chat-utils";

interface StreamingFooterProps {
  sessionId: string;
  completedSegments: StreamSegment[];
  currentContent: string;
  currentThinking: string;
  currentTools: ToolActivity[];
  isStreaming: boolean;
  streamStartedAt: number | null;
  liveTokenCount: number;
  error?: string;
  isConnectionError?: boolean;
  onRetry?: () => void;
  onFilePreview?: (path: string) => void;
}

export function StreamingFooter({
  sessionId, completedSegments, currentContent, currentThinking,
  currentTools, isStreaming, streamStartedAt, liveTokenCount,
  error, isConnectionError, onRetry, onFilePreview,
}: StreamingFooterProps) {
  const { isCompressing } = useCompression(sessionId);
  const loadingStartedAt = streamStartedAt ?? Date.now();

  return (
    <div className="chat-msg-item">
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
        <ErrorBubble message={error} isConnection={isConnectionError} onRetry={onRetry} />
      )}
    </div>
  );
}
