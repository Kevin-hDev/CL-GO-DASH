import { MessageList } from "./message-list";
import type { useAgentChat } from "@/hooks/use-agent-chat";
import type { useChatViewRuntime } from "@/hooks/use-chat-view-runtime";
import type { SubagentInfo } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";

interface ChatMessagePanelProps {
  sessionId: string;
  chat: ReturnType<typeof useAgentChat>;
  runtime: ReturnType<typeof useChatViewRuntime>;
  projectPath?: string;
  knownSubagents: SubagentInfo[];
  cloneEnabled: boolean;
  requestClone: (messageId: string) => void;
  onFilePreviewPath?: (target: string | FileOperation) => void;
  onOpenSubagent?: (sessionId: string) => void;
}

export function ChatMessagePanel({
  sessionId,
  chat,
  runtime,
  projectPath,
  knownSubagents,
  cloneEnabled,
  requestClone,
  onFilePreviewPath,
  onOpenSubagent,
}: ChatMessagePanelProps) {
  return (
    <MessageList
      sessionId={sessionId}
      messages={chat.messages}
      queuedUserMessages={chat.queuedUserMessages}
      completedSegments={chat.completedSegments}
      currentContent={chat.currentContent}
      currentContentPhase={chat.currentContentPhase}
      currentThinking={chat.currentThinking}
      currentTools={chat.currentTools}
      activeStreamItem={chat.activeStreamItem}
      isStreaming={chat.isStreaming}
      tps={chat.tps}
      totalElapsedMs={chat.totalElapsedMs}
      segmentStartedAt={chat.streamStartedAt}
      liveTokenCount={chat.liveTokenCount}
      streamRunId={chat.streamRunId}
      planPreview={chat.planPreview}
      onReload={runtime.handleReload}
      onEdit={runtime.handleEdit}
      onCloneMessage={cloneEnabled ? requestClone : undefined}
      onFileClick={runtime.handleFileClick}
      onFilePreview={onFilePreviewPath}
      projectPath={projectPath}
      onFileReview={onFilePreviewPath}
      knownSubagents={knownSubagents}
      onOpenSubagent={onOpenSubagent}
    />
  );
}
