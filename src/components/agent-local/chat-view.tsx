import { useState, useRef, useEffect } from "react";
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
import { useChatScroll } from "@/hooks/use-chat-scroll";
import { useModelSwitch } from "@/hooks/use-model-switch";
import { PermissionDialog } from "./permission-dialog";
import { TerminalPanel } from "@/components/terminal/terminal-panel";
import { useTerminal } from "@/hooks/use-terminal";
import type { Project } from "@/types/agent";
import scrollDownIcon from "@/assets/fleche.png";
import "./chat.css";

interface ChatViewProps {
  sessionId: string;
  model: string;
  provider: string;
  projects: Project[];
  onAddProject: (path: string) => Promise<Project>;
  onSessionsRefresh?: () => void;
  onApplySwitch?: (model: string, provider: string) => void;
  onNewSession?: (model: string, provider: string) => void;
  onAutoRename?: (id: string, name: string) => void;
  initialMessage?: string;
  initialWorkingDir?: string;
  initialSkills?: { name: string; content: string }[];
  initialFiles?: DroppedFile[];
  thinking: boolean;
  onToggleThinking: () => void;
  onInitialMessageSent?: () => void;
  terminalState: ReturnType<typeof useTerminal>;
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
  onAutoRename,
  initialMessage,
  initialWorkingDir,
  initialSkills,
  initialFiles,
  thinking,
  onToggleThinking,
  onInitialMessageSent,
  terminalState,
}: ChatViewProps) {
  const permissions = usePermissionRequests();
  const permMode = usePermissionMode();
  const chat = useAgentChat(sessionId, model, provider, (id, toolName, args) =>
    permissions.enqueue({ id, toolName, arguments: args }),
    undefined, thinking,
  );
  const fileDrop = useFileDrop();
  const context = useContextProgress(model, chat.tokenCount, provider);
  const [preview, setPreview] = useState<DroppedFile | null>(null);
  const proj = useSessionProject(sessionId, projects, onAddProject, chat.messages.length > 0);
  const initialSent = useRef(false);

  useEffect(() => {
    const hasInitialContent = initialMessage || (initialFiles && initialFiles.length > 0) || (initialSkills && initialSkills.length > 0);
    if (hasInitialContent && !initialSent.current) {
      initialSent.current = true;
      const workingDir = initialWorkingDir ?? proj.selectedProject?.path;
      const files = initialFiles?.map((f) => ({ name: f.name, path: f.path, preview: f.preview }));
      chat.sendMessage(initialMessage || "", files, workingDir, proj.selectedProjectId ?? undefined, initialSkills).then(() => onInitialMessageSent?.());
    }
  }, [initialMessage]);

  const { scrollRef, bottomRef, isAtBottom, scrollToBottom, handleScroll } = useChatScroll({
    messagesLength: chat.messages.length,
    currentContent: chat.currentContent,
    currentThinking: chat.currentThinking,
    currentToolsLength: chat.currentTools.length,
  });

  const { pendingSwitch, setPendingSwitch, handleModelSelect, rememberedRef } = useModelSwitch({
    currentModel: model,
    currentProvider: provider,
    messagesLength: chat.messages.length,
    onApplySwitch,
    onNewSession,
  });

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
            totalElapsedMs={chat.totalElapsedMs}
            segmentStartedAt={chat.segmentStartedAt}
            liveTokenCount={chat.liveTokenCount}
            error={chat.error}
            isConnectionError={chat.isConnectionError}
            onRetry={() => {
              const lastUser = [...chat.messages].reverse().find((m) => m.role === "user");
              if (lastUser) chat.reload(lastUser.id);
            }}
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
              onSend={(text, sentFiles, skills) => {
                const isFirst = chat.messages.length < 1;
                chat.sendMessage(text, sentFiles, proj.selectedProject?.path, proj.selectedProjectId ?? undefined, skills)
                  .then(() => {
                    if (proj.selectedProjectId) onSessionsRefresh?.();
                    if (isFirst && text.trim()) {
                      const autoName = text.slice(0, 40).trim();
                      if (autoName) onAutoRename?.(sessionId, autoName);
                    }
                  });
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
              onToggleThinking={onToggleThinking}
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
        <TerminalPanel
          tabs={terminalState.tabs}
          activeTabId={terminalState.activeTabId}
          isOpen={terminalState.isOpen}
          panelHeight={terminalState.panelHeight}
          onAddTab={terminalState.addTab}
          onCloseTab={terminalState.closeTab}
          onSelectTab={terminalState.setActiveTab}
          onRenameTab={terminalState.renameTab}
          onReorderTabs={terminalState.reorderTabs}
          onTogglePanel={terminalState.togglePanel}
          onPtyReady={terminalState.setPtyId}
          onResize={terminalState.resizePanel}
          onSetMaxHeight={terminalState.setMaxHeight}
          defaultCwd={proj.selectedProject?.path || ""}
        />
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
