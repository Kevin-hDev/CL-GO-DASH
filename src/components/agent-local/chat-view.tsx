import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { DownloadSimple } from "@/components/ui/icons";
import { MessageList } from "./message-list";
import { ChatInput } from "./chat-input";
import { TpsDisplay } from "./tps-display";
import { FileDropZone } from "./file-drop-zone";
import { FilePreview } from "./file-preview";
import { SearchBar } from "./search-bar";
import { useAgentChat } from "@/hooks/use-agent-chat";
import { useOllamaStatus } from "@/hooks/use-ollama-status";
import { useFileDrop, type DroppedFile } from "@/hooks/use-file-drop";
import { useSearchInChat } from "@/hooks/use-search-in-chat";

interface ChatViewProps {
  sessionId: string;
  model: string;
}

export function ChatView({ sessionId, model }: ChatViewProps) {
  const chat = useAgentChat(sessionId, model);
  const ollamaRunning = useOllamaStatus();
  const fileDrop = useFileDrop();
  const search = useSearchInChat(chat.messages);
  const [preview, setPreview] = useState<DroppedFile | null>(null);

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
      onDrop={(fl) => fileDrop.addFiles(fl)}
    >
      <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
        {search.visible && (
          <SearchBar
            query={search.query}
            onChange={search.setQuery}
            matchCount={search.matchCount}
            onClose={search.toggle}
          />
        )}
        <MessageList
          messages={chat.messages}
          streamingContent={chat.streamingContent}
          streamingThinking={chat.streamingThinking}
          isStreaming={chat.isStreaming}
        />
        <div style={{
          display: "flex", alignItems: "center", justifyContent: "space-between",
          padding: "var(--space-xs) var(--space-md)",
        }}>
          <TpsDisplay tps={chat.tps} tokenCount={chat.tokenCount} isStreaming={chat.isStreaming} />
          <button className="msg-action-btn" onClick={handleExport} title="Export Markdown">
            <DownloadSimple size={14} />
          </button>
        </div>
        <ChatInput
          modelName={model}
          ollamaRunning={ollamaRunning}
          isStreaming={chat.isStreaming}
          files={fileDrop.files}
          onSend={chat.sendMessage}
          onStop={chat.stop}
          onFileImport={() => {}}
          onRemoveFile={fileDrop.removeFile}
          onPreviewFile={setPreview}
        />
      </div>
      {preview && (
        <FilePreview
          src={preview.preview ?? preview.name}
          name={preview.name}
          isImage={!!preview.preview}
          onClose={() => setPreview(null)}
        />
      )}
    </FileDropZone>
  );
}
