/* eslint-disable react-hooks/refs -- refs passed as props to JSX ref attribute is valid */
import { MessageList } from "./message-list";
import { ScrollBottomButton } from "./scroll-bottom-button";
import type { ToolActivity, StreamSegment } from "@/hooks/agent-chat-utils";
import type { AgentMessage, TokenPhase } from "@/types/agent";

interface ChatMessagePaneProps {
  sessionId: string;
  scrollRef: React.RefObject<HTMLDivElement | null>;
  bottomRef: React.RefObject<HTMLDivElement | null>;
  isAtBottom: boolean;
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
  onScroll: () => void;
  onScrollToBottom: () => void;
  onReload: (id: string) => void;
  onEdit: (id: string, content: string) => void;
  onFileClick: (file: { name: string; path?: string; thumbnail?: string }) => void;
  onFilePreview?: (path: string) => void;
}

export function ChatMessagePane(props: ChatMessagePaneProps) {
  return (
    <>
      <div className="chat-messages" ref={props.scrollRef} onScroll={props.onScroll}>
        <MessageList
          sessionId={props.sessionId}
          messages={props.messages}
          completedSegments={props.completedSegments}
          currentContent={props.currentContent}
          currentContentPhase={props.currentContentPhase}
          currentThinking={props.currentThinking}
          currentTools={props.currentTools}
          isStreaming={props.isStreaming}
          tps={props.tps}
          totalElapsedMs={props.totalElapsedMs}
          segmentStartedAt={props.segmentStartedAt}
          liveTokenCount={props.liveTokenCount}
          onReload={props.onReload}
          onEdit={props.onEdit}
          onFileClick={props.onFileClick}
          onFilePreview={props.onFilePreview}
        />
        <div ref={props.bottomRef} />
      </div>
      {!props.isAtBottom && <ScrollBottomButton onClick={props.onScrollToBottom} />}
    </>
  );
}
