import { useState, useEffect, useCallback, useMemo } from "react";
import { MessageList } from "./message-list";
import { ChatInput } from "./chat-input";
import { ErrorBubble } from "./error-bubble";
import { FileDropZone } from "./file-drop-zone";
import { ChatOverlays } from "./chat-overlays";
import { SubagentAccordion } from "./subagent-accordion";
import { TodoProgressPanel } from "./todo-progress-panel";
import { ChatProjectControls } from "./chat-project-controls";
import { useAgentChat } from "@/hooks/use-agent-chat";
import { useContextProgress } from "@/hooks/use-context-progress";
import { useContextUsage } from "@/hooks/use-context-usage";
import { useFileDrop, type DroppedFile } from "@/hooks/use-file-drop";
import { usePermissionMode } from "@/hooks/use-permission-mode";
import { usePermissionRequests } from "@/hooks/use-permission-requests";
import { useSessionProject } from "@/hooks/use-session-project";
import { useChatScroll } from "@/hooks/use-chat-scroll";
import { useModelSwitch } from "@/hooks/use-model-switch";
import { useWorktreeSessionSwitch } from "@/hooks/use-worktree-session-switch";
import { useSessionFileGroups } from "@/hooks/use-session-files";
import { useSubagents } from "@/hooks/use-subagents";
import { useSubagentSynthesis } from "@/hooks/use-subagent-synthesis";
import { useChatActions } from "@/hooks/use-chat-actions";
import { useAvailableModels } from "@/hooks/use-available-models";
import { useOllamaConnectionRetry } from "@/hooks/use-ollama-connection-retry";
import { PermissionDialog } from "./permission-dialog";
import { TerminalPanel } from "@/components/terminal/terminal-panel";
import type { useTerminal } from "@/hooks/use-terminal";
import type { Project } from "@/types/agent";
import type { FileOperationGroups } from "@/types/file-preview";
import type { ReasoningMode } from "@/lib/reasoning-modes";
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
  onNewSessionInProject?: (model: string, provider: string, projectId: string) => void;
  onAutoRename?: (id: string, name: string) => void;
  initialMessage?: string;
  initialWorkingDir?: string;
  initialSkills?: { name: string; content: string }[];
  initialFiles?: DroppedFile[];
  reasoningMode?: string | null;
  onReasoningModeChange: (mode: ReasoningMode) => void;
  onInitialMessageSent?: () => void;
  terminalState: ReturnType<typeof useTerminal>;
  onFileOperationsChange?: (operations: FileOperationGroups) => void;
  onFilePreviewPath?: (path: string) => void;
  onOpenSubagent?: (sessionId: string) => void;
  isSubagent?: boolean;
}
export function ChatView({
  sessionId, model, provider, projects, onAddProject,
  onSessionsRefresh, onApplySwitch, onNewSession, onNewSessionInProject, onAutoRename,
  initialMessage, initialWorkingDir, initialSkills, initialFiles,
  reasoningMode, onReasoningModeChange, onInitialMessageSent,
  terminalState, onFileOperationsChange, onFilePreviewPath,
  onOpenSubagent, isSubagent = false,
}: ChatViewProps) {
  const permissions = usePermissionRequests();
  const permMode = usePermissionMode(sessionId);
  const { groups: availableModels } = useAvailableModels();
  const selectedModelCaps = useMemo(
    () => availableModels.get(provider)?.find((entry) => entry.id === model) ?? null,
    [availableModels, provider, model],
  );
  const chat = useAgentChat(sessionId, model, provider, (id, toolName, args) =>
    permissions.enqueue({ id, toolName, arguments: args }),
    selectedModelCaps?.supports_tools,
    selectedModelCaps?.supports_thinking,
    selectedModelCaps?.supports_vision,
    reasoningMode,
    permMode.mode,
  );
  const subagents = useSubagents(isSubagent ? undefined : sessionId);
  const fileDrop = useFileDrop();
  const context = useContextProgress(model, chat.sessionTokenCount, provider);
  const [preview, setPreview] = useState<DroppedFile | null>(null);
  const proj = useSessionProject(sessionId, projects, onAddProject, chat.messages.length > 0);
  const contextUsage = useContextUsage({
    sessionId, model, provider, messages: chat.messages, used: context.used,
    workingDir: proj.selectedProject?.path, permissionMode: permMode.mode,
    planMode: chat.planModeEnabled, supportsTools: selectedModelCaps?.supports_tools,
  });
  const git = useGitBranch(proj.selectedProject?.path, sessionId);
  const fileOperations = useSessionFileGroups(
    chat.messages,
    chat.completedSegments,
    chat.currentTools,
    proj.selectedProject?.path,
  );
  useEffect(() => {
    onFileOperationsChange?.(fileOperations);
  }, [fileOperations, onFileOperationsChange]);
  useSubagentSynthesis({
    parentSessionId: sessionId, allDone: subagents.allDone,
    runId: subagents.doneRunId, isStreaming: chat.isStreaming,
    onStarted: subagents.clearSynthesisSignal,
  });
  const { handleSend, handleFileImport } = useChatActions({
    chat, selectedProjectPath: proj.selectedProject?.path,
    selectedProjectId: proj.selectedProjectId ?? undefined,
    onSessionsRefresh, onAutoRename, sessionId,
    initialMessage, initialWorkingDir, initialSkills, initialFiles,
    onInitialMessageSent, fileDrop,
  });
  const { containerRef, isAtBottom, scrollToBottom } = useChatScroll(
    sessionId, chat.isStreaming,
    [chat.currentContent, chat.currentContentPhase, chat.currentThinking, chat.completedSegments, chat.messages, chat.planPreview],
  );
  const { messages, reload } = chat;
  const handleRetry = useCallback(() => {
    const u = [...messages].reverse().find((m) => m.role === "user");
    if (u) void reload(u.id);
  }, [messages, reload]);
  const connectionRetry = useOllamaConnectionRetry({
    error: chat.error,
    isConnectionError: chat.isConnectionError,
    isStreaming: chat.isStreaming,
    onRetry: handleRetry,
  });
  const retryIndicator = chat.retryIndicator ?? connectionRetry.indicator;
  const showError = !!chat.error && !chat.isStreaming && !connectionRetry.suppressError;
  const handleReload = useCallback((id: string) => void chat.reload(id), [chat]);
  const handleEdit = useCallback((id: string, c: string) => void chat.edit(id, c), [chat]);
  const handleFileClick = useCallback((f: { name: string; path?: string; thumbnail?: string }) => {
    setPreview({ name: f.name, path: f.path, type: "", size: 0, preview: f.thumbnail });
  }, []);
  const { pendingSwitch, setPendingSwitch, handleModelSelect, rememberedRef } = useModelSwitch({
    currentModel: model, currentProvider: provider,
    messagesLength: chat.messages.length, onApplySwitch, onNewSession,
  });
  const worktreeSwitch = useWorktreeSessionSwitch({
    projects, model, provider, onAddProject, onNewSessionInProject,
  });
  return (
    <FileDropZone dragging={fileDrop.dragging} onDragChange={fileDrop.setDragging} onDropPaths={(paths) => void fileDrop.addByPaths(paths)}>
      <div className="chat-zone" style={{ opacity: chat.sessionLoading ? 0 : 1 }}>
        <div className="chat-messages" ref={containerRef}>
          <MessageList
            sessionId={sessionId} messages={chat.messages} completedSegments={chat.completedSegments}
            currentContent={chat.currentContent} currentContentPhase={chat.currentContentPhase}
            currentThinking={chat.currentThinking} currentTools={chat.currentTools}
            isStreaming={chat.isStreaming} tps={chat.tps} totalElapsedMs={chat.totalElapsedMs}
            segmentStartedAt={chat.streamStartedAt} liveTokenCount={chat.liveTokenCount}
            planPreview={chat.planPreview}
            onReload={handleReload} onEdit={handleEdit}
            onFileClick={handleFileClick} onFilePreview={onFilePreviewPath} projectPath={proj.selectedProject?.path}
            completedSubagents={subagents.completed.length > 0 ? subagents.completed : undefined}
            onOpenSubagent={onOpenSubagent}
          />
        </div>
        <div className="chat-input-area">
          <div className="chat-input-column">
            <TodoProgressPanel sessionId={isSubagent ? undefined : sessionId} />
            {subagents.active.length > 0 && (
              <SubagentAccordion
                subagents={subagents.active}
                onCancel={(id) => void subagents.cancelSubagent(id)}
                onOpen={(id) => onOpenSubagent?.(id)}
              />
            )}
            {permissions.current && (
              <PermissionDialog request={permissions.current} onDecide={(id, decision) => void permissions.respond(id, decision)} />
            )}
            {showError && chat.error && (
              <ErrorBubble
                message={chat.error}
                isConnection={chat.isConnectionError}
                diagnosticSummary={chat.diagnosticSummary}
                onRetry={handleRetry}
              />
            )}
            <ChatInput
              modelName={model} providerName={provider} isStreaming={chat.isStreaming} reasoningMode={reasoningMode}
              files={fileDrop.files} contextUsed={contextUsage.used} contextMax={context.max} contextBreakdown={contextUsage}
              retryIndicator={retryIndicator}
              interactiveRequest={chat.interactiveChoice}
              onInteractiveResolved={chat.clearInteractiveChoice}
              permissionMode={permMode.mode} onPermissionModeChange={(m) => void permMode.change(m)}
              planModeEnabled={chat.planModeEnabled}
              onPlanModeChange={(enabled) => void chat.setPlanModeEnabled(enabled)}
              onRemoveFile={fileDrop.removeFile} onPreviewFile={setPreview} onSend={handleSend}
              onStop={() => void chat.stop()} onClearFiles={fileDrop.clearFiles} onFileImport={handleFileImport}
              onModelChange={handleModelSelect} onReasoningModeChange={onReasoningModeChange}
            />
            <div className="chat-input-under-row">
              <div className="chat-input-project-slot">
                <ChatProjectControls
                  projects={projects}
                  projectState={proj}
                  git={git}
                  onWorktreeSelect={worktreeSwitch.request}
                />
              </div>
              {!isAtBottom && <ScrollBottomButton variant="inline" onClick={scrollToBottom} />}
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
      <ChatOverlays
        preview={preview} currentModel={model} pendingSwitch={pendingSwitch}
        pendingWorktreeSwitch={worktreeSwitch.pending}
        onClosePreview={() => setPreview(null)} onCancelSwitch={() => setPendingSwitch(null)}
        onCancelWorktreeSwitch={worktreeSwitch.cancel}
        onNewSession={(remember) => { if (remember) rememberedRef.current = "new"; onNewSession?.(pendingSwitch!.model, pendingSwitch!.provider); setPendingSwitch(null); }}
        onContinue={(remember) => { if (remember) rememberedRef.current = "continue"; onApplySwitch?.(pendingSwitch!.model, pendingSwitch!.provider); setPendingSwitch(null); }}
        onNewWorktreeSession={() => void worktreeSwitch.createSession()}
      />
    </FileDropZone>
  );
}
