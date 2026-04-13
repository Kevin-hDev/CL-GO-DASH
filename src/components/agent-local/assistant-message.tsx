import { CodeBlock } from "./code-block";
import { ThinkingSection } from "./thinking-section";
import { MessageActions } from "./message-actions";
import { SavedToolBubble } from "./tool-bubble";
import type { ToolActivityRecord } from "@/types/agent";
import "./messages.css";

interface AssistantMessageProps {
  content: string;
  thinking?: string;
  toolActivities?: ToolActivityRecord[];
  isStreaming?: boolean;
  onReload?: () => void;
}

export function AssistantMessage({
  content, thinking, toolActivities, isStreaming, onReload,
}: AssistantMessageProps) {
  return (
    <div className="msg-assistant">
      {thinking && <ThinkingSection content={thinking} />}
      {toolActivities && toolActivities.length > 0 && (
        <SavedToolBubble tools={toolActivities} />
      )}
      <div style={{ display: "flex", gap: 4, alignItems: "flex-start" }}>
        <div className="msg-assistant-content">
          {renderMarkdown(content)}
          {isStreaming && <span style={{ animation: "pulse-dot 1s infinite" }}>▊</span>}
        </div>
        {!isStreaming && (
          <MessageActions role="assistant" content={content} onReload={onReload} />
        )}
      </div>
    </div>
  );
}

function renderMarkdown(text: string) {
  const parts: React.ReactNode[] = [];
  const codeBlockRegex = /```(\w*)\n([\s\S]*?)```/g;
  let lastIndex = 0;
  let match;

  while ((match = codeBlockRegex.exec(text)) !== null) {
    if (match.index > lastIndex) {
      parts.push(<span key={lastIndex}>{text.slice(lastIndex, match.index)}</span>);
    }
    parts.push(<CodeBlock key={match.index} language={match[1]} code={match[2]} />);
    lastIndex = match.index + match[0].length;
  }

  if (lastIndex < text.length) {
    parts.push(<span key={lastIndex}>{text.slice(lastIndex)}</span>);
  }

  return parts.length > 0 ? parts : text;
}
