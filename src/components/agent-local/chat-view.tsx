import { useState, useRef, useCallback, useEffect } from "react";
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";
import { MessageList } from "./message-list";
import { ChatInput } from "./chat-input";
import { ProjectSelector } from "./project-selector";
import { FileDropZone } from "./file-drop-zone";
import { FilePreview } from "./file-preview";
import { SwitchModelDialog } from "./switch-model-dialog";
import { useAgentChat } from "@/hooks/use-agent-chat";
import { useContextProgress } from "@/hooks/use-context-progress";
import { useFileDrop, type DroppedFile } from "@/hooks/use-file-drop";
import { usePermissionMode } from "@/hooks/use-permission-mode";
import { usePermissionRequests } from "@/hooks/use-permission-requests";
import { useSessionProject } from "@/hooks/use-session-project";
import { PermissionDialog } from "./permission-dialog";
import type { Project } from "@/types/agent";
import scrollDownIcon from "@/assets/fleche.png";
import "./chat.css";

type SwitchChoice = "new" | "continue";

interface ChatViewProps {
  sessionId: string;
  model: string;
  provider: string;
  projects: Project[];
  onAddProject: (path: string) => Promise<Project>;
  onSessionsRefresh?: () => void;
  onApplySwitch?: (model: string, provider: string) => void;
  onNewSession?: (model: string, provider: string) => void;
  initialMessage?: string;
  initialWorkingDir?: string;
  initialSkillContent?: string;
  initialSkillName?: string;
  onInitialMessageSent?: () => void;
}

interface PendingSwitch {
  model: string;
  provider: string;
}

export function ChatView({
  sessionId,
  model,
  provider,
  projects,
  onAddProject,
  onSessionsRefresh,
  onApplySwitch,
  onNewSession,
  initialMessage,
  initialWorkingDir,
  initialSkillContent,
  initialSkillName,
  onInitialMessageSent,
}: ChatViewProps) {
  const permissions = usePermissionRequests();
  const permMode = usePermissionMode();
  const chat = useAgentChat(sessionId, model, provider, (id, toolName, args) =>
    permissions.enqueue({ id, toolName, arguments: args }),
  );
  const fileDrop = useFileDrop();
  const context = useContextProgress(model, chat.tokenCount, provider);
  const [preview, setPreview] = useState<DroppedFile | null>(null);
  const [thinking, setThinking] = useState(false);
  const [pendingSwitch, setPendingSwitch] = useState<PendingSwitch | null>(null);
  const rememberedRef = useRef<SwitchChoice | null>(null);
  const proj = useSessionProject(sessionId, projects, onAddProject, chat.messages.length > 0);
  const initialSent = useRef(false);

  useEffect(() => {
    if ((initialMessage || initialSkillContent) && !initialSent.current) {
      initialSent.current = true;
      const workingDir = initialWorkingDir ?? proj.selectedProject?.path;
      chat.sendMessage(initialMessage || "", [], workingDir, proj.selectedProjectId ?? undefined, initialSkillContent, initialSkillName).then(() => onInitialMessageSent?.());
    }
  }, [initialMessage]);

  const scrollRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const [isAtBottom, setIsAtBottom] = useState(true);
  const following = useRef(true);

  const handleScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 80;
    setIsAtBottom(atBottom);
    if (atBottom) following.current = true;
  }, []);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      if (e.deltaY < 0) following.current = false;
    };
    el.addEventListener("wheel", onWheel, { passive: true });
    return () => el.removeEventListener("wheel", onWheel);
  }, []);

  const scrollToBottom = useCallback(() => {
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
    following.current = true;
  }, []);

  useEffect(() => {
    following.current = true;
  }, [chat.messages.length]);

  useEffect(() => {
    if (!following.current) return;
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [chat.currentContent]);

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
            tps={chat.tps}
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
              isStreaming={chat.isStreaming}
              thinkingEnabled={thinking}
              files={fileDrop.files}
              contextUsed={context.used}
              contextMax={context.max}
              permissionMode={permMode.mode}
              onPermissionModeChange={permMode.change}
              onRemoveFile={fileDrop.removeFile}
              onPreviewFile={setPreview}
              onSend={(text, sentFiles, skillContent, skillName) => {
                chat.sendMessage(text, sentFiles, proj.selectedProject?.path, proj.selectedProjectId ?? undefined, skillContent, skillName)
                  .then(() => { if (proj.selectedProjectId) onSessionsRefresh?.(); });
              }}
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
            />
            <ProjectSelector
              projects={projects}
              selectedProjectId={proj.selectedProjectId}
              locked={proj.locked}
              hidden={proj.hidden}
              onSelect={proj.setSelectedProjectId}
              onAddProject={proj.handleAddProject}
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
