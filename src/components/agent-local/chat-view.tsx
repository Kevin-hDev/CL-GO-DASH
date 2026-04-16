import { useState, useRef, useCallback } from "react";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { MessageList } from "./message-list";
import { ChatInput } from "./chat-input";
import { FileDropZone } from "./file-drop-zone";
import { FilePreview } from "./file-preview";
import { SwitchModelDialog } from "./switch-model-dialog";
import { useAgentChat } from "@/hooks/use-agent-chat";
import { useOllamaStatus } from "@/hooks/use-ollama-status";
import { useContextProgress } from "@/hooks/use-context-progress";
import { useFileDrop, type DroppedFile } from "@/hooks/use-file-drop";
import { usePermissionMode } from "@/hooks/use-permission-mode";
import { usePermissionRequests } from "@/hooks/use-permission-requests";
import { PermissionDialog } from "./permission-dialog";
import scrollDownIcon from "@/assets/fleche.png";
import "./chat.css";

type SwitchChoice = "new" | "continue";

interface ChatViewProps {
  sessionId: string;
  model: string;
  provider: string;
  /** Applique le changement dans la session courante (model + provider). */
  onApplySwitch?: (model: string, provider: string) => void;
  /** Crée une nouvelle session avec le nouveau modèle/provider. */
  onNewSession?: (model: string, provider: string) => void;
}

interface PendingSwitch {
  model: string;
  provider: string;
}

export function ChatView({
  sessionId,
  model,
  provider,
  onApplySwitch,
  onNewSession,
}: ChatViewProps) {
  const permissions = usePermissionRequests();
  const permMode = usePermissionMode();
  const chat = useAgentChat(sessionId, model, provider, (id, toolName, args) =>
    permissions.enqueue({ id, toolName, arguments: args }),
  );
  const ollamaRunning = useOllamaStatus();
  const fileDrop = useFileDrop();
  const context = useContextProgress(model, chat.tokenCount);
  const [preview, setPreview] = useState<DroppedFile | null>(null);
  const [thinking, setThinking] = useState(false);
  const [pendingSwitch, setPendingSwitch] = useState<PendingSwitch | null>(null);
  const rememberedRef = useRef<SwitchChoice | null>(null);

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

  const handleModelSelect = useCallback(
    (newModel: string, newProvider: string) => {
      if (newModel === model && newProvider === provider) return;
      const hasMessages = chat.messages.length > 0;
      if (!hasMessages) {
        onApplySwitch?.(newModel, newProvider);
        return;
      }
      if (rememberedRef.current === "continue") {
        onApplySwitch?.(newModel, newProvider);
        return;
      }
      if (rememberedRef.current === "new") {
        onNewSession?.(newModel, newProvider);
        return;
      }
      setPendingSwitch({ model: newModel, provider: newProvider });
    },
    [model, provider, chat.messages.length, onApplySwitch, onNewSession],
  );

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
          <div className="chat-input-column">
            {permissions.current && (
              <PermissionDialog
                request={permissions.current}
                onDecide={permissions.respond}
              />
            )}
            <ChatInput
              modelName={model}
              providerName={provider}
              ollamaRunning={ollamaRunning}
              isStreaming={chat.isStreaming}
              thinkingEnabled={thinking}
              files={fileDrop.files}
              contextUsed={context.used}
              contextMax={context.max}
              tps={chat.tps}
              permissionMode={permMode.mode}
              onPermissionModeChange={permMode.change}
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
              onModelChange={handleModelSelect}
              onToggleThinking={() => setThinking(!thinking)}
              onSkillLoaded={chat.setSkill}
            />
          </div>
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
      {pendingSwitch && (
        <SwitchModelDialog
          fromModel={model}
          toModel={pendingSwitch.model}
          onNewSession={(remember) => {
            if (remember) rememberedRef.current = "new";
            onNewSession?.(pendingSwitch.model, pendingSwitch.provider);
            setPendingSwitch(null);
          }}
          onContinue={(remember) => {
            if (remember) rememberedRef.current = "continue";
            onApplySwitch?.(pendingSwitch.model, pendingSwitch.provider);
            setPendingSwitch(null);
          }}
          onCancel={() => setPendingSwitch(null)}
        />
      )}
    </FileDropZone>
  );
}
