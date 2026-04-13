import { useState, useRef, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { DownloadSimple } from "@/components/ui/icons";
import { MessageList } from "./message-list";
import { ChatInput } from "./chat-input";
import { TpsDisplay } from "./tps-display";
import { FileDropZone } from "./file-drop-zone";
import { FilePreview } from "./file-preview";
import { useAgentChat } from "@/hooks/use-agent-chat";
import { useOllamaStatus } from "@/hooks/use-ollama-status";
import { useFileDrop, type DroppedFile } from "@/hooks/use-file-drop";
import scrollDownIcon from "@/assets/fleche.png";
import "./chat.css";

interface ChatViewProps {
  sessionId: string;
  model: string;
  onModelChange?: (model: string) => void;
  onTokenCountChange?: (count: number) => void;
}

export function ChatView({ sessionId, model, onModelChange, onTokenCountChange }: ChatViewProps) {
  const chat = useAgentChat(sessionId, model);
  const ollamaRunning = useOllamaStatus();
  const fileDrop = useFileDrop();
  const [preview, setPreview] = useState<DroppedFile | null>(null);
  const [thinking, setThinking] = useState(false);

  useEffect(() => {
    onTokenCountChange?.(chat.tokenCount);
  }, [chat.tokenCount, onTokenCountChange]);

  const scrollRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const [isAtBottom, setIsAtBottom] = useState(true);

  const handleScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    setIsAtBottom(el.scrollHeight - el.scrollTop - el.clientHeight < 80);
  }, []);

  const scrollToBottom = useCallback(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, []);

  const handleExport = useCallback(async () => {
    try {
      const md = await invoke<string>("export_agent_session_markdown", { id: sessionId });
      const blob = new Blob([md], { type: "text/markdown" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `conversation-${sessionId.slice(0, 8)}.md`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e: unknown) {
      console.warn("Export erreur:", e);
    }
  }, [sessionId]);

  return (
    <FileDropZone
      dragging={fileDrop.dragging}
      onDragChange={fileDrop.setDragging}
      onDropPaths={(paths) => fileDrop.addByPaths(paths)}
    >
      <div className="chat-zone">
        <div className="chat-messages" ref={scrollRef} onScroll={handleScroll}>
          <MessageList
            messages={chat.messages}
            completedSegments={chat.completedSegments}
            currentContent={chat.currentContent}
            currentThinking={chat.currentThinking}
            currentTools={chat.currentTools}
            isStreaming={chat.isStreaming}
            onReload={chat.reload}
            onEdit={chat.edit}
            onFileClick={(f) => setPreview({
              name: f.name,
              path: f.path,
              type: "",
              size: 0,
              preview: f.thumbnail,
            })}
          />
          <div ref={bottomRef} />
        </div>

        {!isAtBottom && (
          <button className="scroll-bottom-btn" onClick={scrollToBottom}>
            <img src={scrollDownIcon} alt="" style={{ width: 20, height: 20 }} />
          </button>
        )}

        <div className="chat-input-area">
          <div style={{ display: "flex", alignItems: "center", gap: 4 }}>
            <TpsDisplay
              tps={chat.tps}
              tokenCount={chat.tokenCount}
              isStreaming={chat.isStreaming}
            />
            {chat.messages.length > 0 && (
              <button
                className="msg-action-btn"
                onClick={handleExport}
                title="Export Markdown"
              >
                <DownloadSimple size={14} />
              </button>
            )}
          </div>
          <ChatInput
            modelName={model}
            ollamaRunning={ollamaRunning}
            isStreaming={chat.isStreaming}
            thinkingEnabled={thinking}
            files={fileDrop.files}
            onRemoveFile={fileDrop.removeFile}
            onPreviewFile={setPreview}
            onSend={(text, sentFiles) => chat.sendMessage(text, sentFiles)}
            onStop={chat.stop}
            onClearFiles={fileDrop.clearFiles}
            onFileImport={async () => {
              const result = await openFileDialog({ multiple: true });
              if (!result) return;
              const raw = Array.isArray(result) ? result : [result];
              const paths = raw.map((p) => String(p));
              fileDrop.addByPaths(paths);
            }}
            onModelChange={(m) => onModelChange?.(m)}
            onToggleThinking={() => setThinking(!thinking)}
            onSkillLoaded={chat.setSkill}
          />
        </div>
      </div>
      {preview && (
        <FilePreview
          name={preview.name}
          path={preview.path}
          thumbnail={preview.preview}
          isImage={!!preview.preview}
          onClose={() => setPreview(null)}
        />
      )}
    </FileDropZone>
  );
}
