import { useState, useEffect, useCallback } from "react";
import { MessageList } from "./message-list";
import { ChatInput } from "./chat-input";
import { ProjectSelector } from "./project-selector";
import { FileDropZone } from "./file-drop-zone";
import { ChatOverlays } from "./chat-overlays";
import { SubagentAccordion } from "./subagent-accordion";
import { useAgentChat } from "@/hooks/use-agent-chat";
import { useContextProgress } from "@/hooks/use-context-progress";
import { useFileDrop, type DroppedFile } from "@/hooks/use-file-drop";
import { usePermissionMode } from "@/hooks/use-permission-mode";
import { usePermissionRequests } from "@/hooks/use-permission-requests";
import { useSessionProject } from "@/hooks/use-session-project";
import { useChatScroll } from "@/hooks/use-chat-scroll";
import { useModelSwitch } from "@/hooks/use-model-switch";
import { useSessionFiles } from "@/hooks/use-session-files";
import { useSubagents } from "@/hooks/use-subagents";
import { useSubagentSynthesis } from "@/hooks/use-subagent-synthesis";
import { useChatActions } from "@/hooks/use-chat-actions";
import { PermissionDialog } from "./permission-dialog";
import { BranchSelector } from "./branch-selector";
import { BranchConflictDialog } from "./branch-conflict-dialog";
import { TerminalPanel } from "@/components/terminal/terminal-panel";
import type { useTerminal } from "@/hooks/use-terminal";
import type { Project } from "@/types/agent";
import type { FileOperation } from "@/types/file-preview";
import { useGitBranch } from "@/hooks/use-git-branch";
import { ScrollBottomButton } from "./scroll-bottom-button";
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
  onFileOperationsChange?: (operations: FileOperation[]) => void;
  onFilePreviewPath?: (path: string) => void;
  onOpenSubagent?: (sessionId: string) => void;
  isSubagent?: boolean;
}

export function ChatView({
  sessionId, model, provider, projects, onAddProject,
  onSessionsRefresh, onApplySwitch, onNewSession, onAutoRename,
  initialMessage, initialWorkingDir, initialSkills, initialFiles,
  thinking, onToggleThinking, onInitialMessageSent,
  terminalState, onFileOperationsChange, onFilePreviewPath,
  onOpenSubagent, isSubagent = false,
}: ChatViewProps) {
  const permissions = usePermissionRequests();
  const permMode = usePermissionMode(sessionId);
  const chat = useAgentChat(sessionId, model, provider, (id, toolName, args) =>
    permissions.enqueue({ id, toolName, arguments: args }),
    undefined, thinking, permMode.mode,
  );
  const subagents = useSubagents(isSubagent ? undefined : sessionId);
  const fileDrop = useFileDrop();
  const context = useContextProgress(model, chat.tokenCount, provider);
  const [preview, setPreview] = useState<DroppedFile | null>(null);
  const proj = useSessionProject(sessionId, projects, onAddProject, chat.messages.length > 0);
  const git = useGitBranch(proj.selectedProject?.path);
  const [branchConflict, setBranchConflict] = useState<{ branch: string; dirtyCount: number } | null>(null);
  const fileOperations = useSessionFiles(chat.messages);

  useEffect(() => {
    onFileOperationsChange?.(fileOperations);
  }, [fileOperations, onFileOperationsChange]);

  useSubagentSynthesis({
    parentSessionId: sessionId, allDone: subagents.allDone,
    runId: subagents.doneRunId, isStreaming: chat.isStreaming,
    onStarted: subagents.clearSynthesisSignal,
  });

  const { handleSend, handleBuiltInCommand, handleFileImport } = useChatActions({
    chat, selectedProjectPath: proj.selectedProject?.path,
    selectedProjectId: proj.selectedProjectId ?? undefined,
    onSessionsRefresh, onAutoRename, sessionId,
    initialMessage, initialWorkingDir, initialSkills, initialFiles,
    onInitialMessageSent, fileDrop,
  });

  const { containerRef, isAtBottom, scrollToBottom } = useChatScroll(
    sessionId, chat.isStreaming,
    [chat.currentContent, chat.currentThinking, chat.completedSegments, chat.messages],
  );

  const handleRetry = useCallback(() => {
    const u = [...chat.messages].reverse().find((m) => m.role === "user");
    if (u) void chat.reload(u.id);
  }, [chat.messages, chat.reload]);

  const handleReload = useCallback((id: string) => void chat.reload(id), [chat.reload]);
  const handleEdit = useCallback((id: string, c: string) => void chat.edit(id, c), [chat.edit]);
  const handleFileClick = useCallback((f: { name: string; path?: string; thumbnail?: string }) => {
    setPreview({ name: f.name, path: f.path, type: "", size: 0, preview: f.thumbnail });
  }, []);

  const { pendingSwitch, setPendingSwitch, handleModelSelect, rememberedRef } = useModelSwitch({
    currentModel: model, currentProvider: provider,
    messagesLength: chat.messages.length, onApplySwitch, onNewSession,
  });

  return (
    <FileDropZone dragging={fileDrop.dragging} onDragChange={fileDrop.setDragging} onDropPaths={(paths) => void fileDrop.addByPaths(paths)}>
      <div className="chat-zone" style={{ opacity: chat.sessionLoading ? 0 : 1 }}>
        <div className="chat-messages" ref={containerRef}>
          <MessageList
            sessionId={sessionId} messages={chat.messages} completedSegments={chat.completedSegments}
            currentContent={chat.currentContent} currentThinking={chat.currentThinking} currentTools={chat.currentTools}
            isStreaming={chat.isStreaming} tps={chat.tps} totalElapsedMs={chat.totalElapsedMs}
            segmentStartedAt={chat.streamStartedAt} liveTokenCount={chat.liveTokenCount}
            error={chat.error} isConnectionError={chat.isConnectionError}
            onRetry={handleRetry}
            onReload={handleReload} onEdit={handleEdit}
            onFileClick={handleFileClick}
            onFilePreview={onFilePreviewPath}
            completedSubagents={subagents.completed.length > 0 ? subagents.completed : undefined}
            onOpenSubagent={onOpenSubagent}
          />
        </div>
        {!isAtBottom && <ScrollBottomButton onClick={scrollToBottom} />}
        <div className="chat-input-area">
          <div className="chat-input-column">
            {subagents.active.length > 0 && (
              <SubagentAccordion subagents={subagents.active} onCancel={subagents.cancelSubagent} onOpen={(id) => onOpenSubagent?.(id)} />
            )}
            {permissions.current && (
              <PermissionDialog request={permissions.current} onDecide={(id, decision) => void permissions.respond(id, decision)} />
            )}
            <ChatInput
              modelName={model} providerName={provider} isStreaming={chat.isStreaming} thinkingEnabled={thinking}
              files={fileDrop.files} contextUsed={context.used} contextMax={context.max}
              permissionMode={permMode.mode} onPermissionModeChange={(m) => void permMode.change(m)}
              onRemoveFile={fileDrop.removeFile} onPreviewFile={setPreview} onSend={handleSend}
              onStop={() => void chat.stop()} onClearFiles={fileDrop.clearFiles} onFileImport={handleFileImport}
              onModelChange={handleModelSelect} onToggleThinking={onToggleThinking} onBuiltInCommand={handleBuiltInCommand}
            />
            <div style={{ display: "flex", alignItems: "center", gap: "var(--space-xs)", flexWrap: "wrap" }}>
              <ProjectSelector
                projects={projects} selectedProjectId={proj.selectedProjectId} locked={proj.locked} hidden={proj.hidden}
                onSelect={proj.setSelectedProjectId} onAddProject={() => void proj.handleAddProject()}
              />
              <BranchSelector
                git={git}
                locked={false}
                onConflict={(branch, dirtyCount) => setBranchConflict({ branch, dirtyCount })}
                onCreateRequest={() => {}}
              />
            </div>
          </div>
        </div>
        <TerminalPanel
          tabs={terminalState.tabs} activeTabId={terminalState.activeTabId} allTabs={terminalState.allTabs()}
          activeGroupKey={terminalState.groupKey} isOpen={terminalState.isOpen} panelHeight={terminalState.panelHeight}
          onAddTab={terminalState.addTab} onCloseTab={terminalState.closeTab} onSelectTab={terminalState.setActiveTab}
          onRenameTab={terminalState.renameTab} onReorderTabs={terminalState.reorderTabs} onTogglePanel={terminalState.togglePanel}
          onPtyReady={terminalState.setPtyId} onResize={terminalState.resizePanel} onSetMaxHeight={terminalState.setMaxHeight}
        />
      </div>
      {branchConflict && proj.selectedProject && (
        <BranchConflictDialog
          targetBranch={branchConflict.branch}
          dirtyCount={branchConflict.dirtyCount}
          projectPath={proj.selectedProject.path}
          onCancel={() => setBranchConflict(null)}
          onCommitAndSwitch={() => setBranchConflict(null)}
        />
      )}
      <ChatOverlays
        preview={preview} currentModel={model} pendingSwitch={pendingSwitch}
        onClosePreview={() => setPreview(null)} onCancelSwitch={() => setPendingSwitch(null)}
        onNewSession={(remember) => { if (remember) rememberedRef.current = "new"; onNewSession?.(pendingSwitch!.model, pendingSwitch!.provider); setPendingSwitch(null); }}
        onContinue={(remember) => { if (remember) rememberedRef.current = "continue"; onApplySwitch?.(pendingSwitch!.model, pendingSwitch!.provider); setPendingSwitch(null); }}
      />
    </FileDropZone>
  );
}
