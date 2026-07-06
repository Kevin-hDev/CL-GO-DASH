import { useCallback, useEffect, useMemo, useRef, useState, memo } from "react";
import { useTranslation } from "react-i18next";
import { ConversationList } from "./conversation-list";
import { ChatHeader } from "./chat-header";
import { AgentChatDetail } from "./agent-chat-detail";
import { WelcomeView } from "./welcome-view";
import { useAgentLocalTab } from "@/hooks/use-agent-local-tab";
import { useFileTree } from "@/hooks/use-file-tree";
import { useForecastPanel } from "@/hooks/use-forecast-panel";
import { useAgentLocalPanelNav } from "@/hooks/use-agent-local-panel-nav";
import { useAgentLocalControlledPanels } from "@/hooks/use-agent-local-controlled-panels";
import { useGitBranch } from "@/hooks/use-git-branch";
import { useSessionSummary } from "@/hooks/use-session-summary";
import { useSessionTabs } from "@/hooks/use-session-tabs";
import { useAgentLocalTabGit } from "@/hooks/use-agent-local-tab-git";
import { ForecastPanel } from "@/components/forecast/forecast-panel";
import { openForecastDocsWindow } from "@/components/forecast/open-forecast-docs";
import { PanelSlot } from "@/components/layout/panel-slots";
import type { AgentLocalTabProps } from "./agent-local-tab-types";
import "./agent-local-tab.css";

export const AgentLocalTab = memo(function AgentLocalTab({
  navState,
  onSessionChange,
  onNavChange,
  listFocused = true,
}: AgentLocalTabProps) {
  const { t } = useTranslation();
  const s = useAgentLocalTab({ navState, onSessionChange, onNavChange, listFocused });
  const { sessions, refresh, rename, updateModel } = s;
  const { projectsHook, terminal, activeSession, activeSessionId } = s;
  const { model, provider, currentDefault, activeProject } = s;
  const { filePreview, fileOperations, setFileOperations } = s;
  const { reasoningMode, setReasoningMode, setWelcomeModel } = s;
  const { sessionActions, handleSelectById, handleDeleteProject, handleDeleteSession } = s;
  const {
    pendingMessage, setPendingMessage,
    pendingWorkingDir, setPendingWorkingDir,
    pendingSkills, setPendingSkills,
    pendingFiles, setPendingFiles,
    handleCreate, handleCreateWithModel,
    handleWelcomeSend, handleAutoRename,
    handleCreateInProject, handleCreateInProjectWithModel,
  } = sessionActions;
  const sessionTabs = useSessionTabs(activeSessionId, refresh);
  const displaySessionId = sessionTabs.activeSessionId ?? activeSessionId;
  const displaySession = displaySessionId
    ? sessions.find((session) => session.id === displaySessionId) ?? activeSession
    : null;
  const displayProject = displaySession?.project_id
    ? projectsHook.projects.find((project) => project.id === displaySession.project_id) ?? activeProject
    : activeProject;
  const displayModel = displaySession?.model ?? model;
  const displayProvider = displaySession?.provider ?? provider;
  const displayReasoningMode = displaySession
    ? displaySession.reasoning_mode ?? (displaySession.thinking_enabled ? "auto" : null)
    : reasoningMode;
  const terminalCwd = displayProject?.path || "";
  const sessionSummary = useSessionSummary(displaySessionId ?? null);
  const summaryGit = useGitBranch(displayProject?.path, displaySessionId ?? undefined);
  const tabGit = useAgentLocalTabGit({
    rootSessionId: activeSessionId,
    git: summaryGit,
    projectPath: displayProject?.path,
    sessionTabs,
  });
  const fileTree = useFileTree(displaySessionId, displayProject?.path);
  const forecast = useForecastPanel(displaySessionId ?? null);
  useAgentLocalPanelNav({ navState, fileTree, forecast });
  const { fileTreeNav, forecastNav } = useAgentLocalControlledPanels({ navState, fileTree, forecast, onNavChange });
  const [fullscreenSwitching, setFullscreenSwitching] = useState(false);
  const fullscreenTimerRef = useRef<number | null>(null);
  const handlePreviewFullscreenChange = useCallback((value: boolean) => {
    if (fullscreenTimerRef.current !== null) window.clearTimeout(fullscreenTimerRef.current);
    setFullscreenSwitching(true);
    filePreview.setFullscreen(value);
    fullscreenTimerRef.current = window.setTimeout(() => setFullscreenSwitching(false), 80);
  }, [filePreview]);

  useEffect(() => () => {
    if (fullscreenTimerRef.current !== null) window.clearTimeout(fullscreenTimerRef.current);
  }, []);

  const handleOpenForecastDocs = useCallback(() => {
    void openForecastDocsWindow(t("forecast.docs.windowTitle")).catch(() => {});
  }, [t]);
  const forecastContent = useMemo(() => (
    <ForecastPanel
      activeSection={forecastNav.activeSection}
      navOpen={forecastNav.navOpen}
      currentAnalysisId={forecastNav.currentAnalysisId}
      fullscreen={filePreview.fullscreen}
      onSectionChange={forecastNav.setSection}
      onToggleNav={forecastNav.toggleNav}
      onLoadAnalysis={forecastNav.loadAnalysis}
      onFocusAnalysis={forecastNav.focusAnalysis}
      onPanelExtraWidthChange={filePreview.setExtraWidth}
      onCloseAnalysis={forecastNav.closeAnalysis}
      onFullscreenChange={handlePreviewFullscreenChange}
    />
  ), [filePreview.fullscreen, filePreview.setExtraWidth, forecastNav, handlePreviewFullscreenChange]);

  const list = useMemo(() => (
    <ConversationList
      sessions={sessions}
      projects={projectsHook.projects}
      selectedId={activeSessionId}
      onSelect={(id) => void handleSelectById(id)}
      onCreate={handleCreate}
      onRename={(id, name) => void rename(id, name)}
      onDelete={(id) => void handleDeleteSession(id)}
      onNewSessionInProject={(pid) => void handleCreateInProject(pid)}
      onRenameProject={(id, name) => void projectsHook.rename(id, name)}
      onDeleteProject={handleDeleteProject}
      onOpenFolder={(path) => void projectsHook.openFolder(path)}
      onReorderProjects={(ids) => void projectsHook.reorder(ids)}
    />
  ), [activeSessionId, handleCreate, handleCreateInProject, handleDeleteProject, handleDeleteSession, handleSelectById, projectsHook, rename, sessions]);

  const detail = useMemo(() => (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
      <div style={{ flexShrink: 0 }}>
        <ChatHeader
          sessionName={activeSession?.name ?? null}
          sessionId={activeSessionId ?? null}
          terminalOpen={terminal.isOpen}
          previewOpen={filePreview.open}
          panelMode={forecastNav.panelMode}
          showForecastDocs={filePreview.open && forecastNav.panelMode === "forecast"}
          onTogglePreview={filePreview.toggleOpen}
          onOpenForecastDocs={handleOpenForecastDocs}
          onPanelModeChange={forecastNav.setPanelMode}
          sessionSummary={sessionSummary}
          summaryGit={summaryGit}
          sessionTabs={sessionTabs.tabs}
          sessionTabAttentionIds={sessionTabs.attentionTabIds}
          onSelectSessionTab={(id) => void tabGit.selectTab(id)}
          onCloseSessionTab={tabGit.closeTab}
          onRenameSessionTab={(id, label) => void sessionTabs.renameTab(id, label)}
          onOpenPlan={filePreview.openPlan}
          onOpenSubagent={(id) => void handleSelectById(id)}
          onToggleTerminal={() => {
            if (!terminal.isOpen && terminal.tabs.length === 0) {
              terminal.addTab(terminalCwd);
            } else {
              terminal.togglePanel();
            }
          }}
        />
      </div>
      {displaySessionId ? (
        <AgentChatDetail
          sessionId={displaySessionId}
          model={displayModel}
          provider={displayProvider}
          projects={projectsHook.projects}
          activeProjectPath={displayProject?.path}
          pendingMessage={pendingMessage}
          pendingWorkingDir={pendingWorkingDir}
          pendingSkills={pendingSkills}
          pendingFiles={pendingFiles}
          reasoningMode={displayReasoningMode}
          terminal={terminal}
          filePreview={filePreview}
          fullscreenSwitching={fullscreenSwitching}
          fileOperations={fileOperations}
          fileTree={fileTreeNav}
          onAddProject={projectsHook.add}
          onSessionsRefresh={() => void refresh()}
          onUpdateModel={(id, m, p) => void updateModel(id, m, p)}
          onNewSession={(m, p) => void handleCreateWithModel(m, p)}
          onNewSessionInProject={(m, p, pid) =>
            void handleCreateInProjectWithModel(m, p, pid)}
          onAutoRename={(id, msg) => void handleAutoRename(id, msg)}
          onReasoningModeChange={setReasoningMode}
          onInitialMessageSent={() => {
            setPendingMessage(null);
            setPendingWorkingDir(undefined);
            setPendingSkills(undefined);
            setPendingFiles(undefined);
          }}
          onFileOperationsChange={setFileOperations}
          onPreviewFullscreenChange={handlePreviewFullscreenChange}
          panelMode={forecastNav.panelMode}
          forecastContent={forecastContent}
          parentSessionId={displaySession?.parent_session_id}
          onOpenSubagent={(id) => void handleSelectById(id)}
          onGoToParent={() => displaySession?.parent_session_id && void handleSelectById(displaySession.parent_session_id)}
          canCloneMessages={!displaySession?.parent_session_id && !displaySession?.clone_parent_session_id}
          onCloneMessage={(messageId, cloneMode, customFocus, options) =>
            sessionTabs.cloneMessage({
              messageId,
              mode: cloneMode,
              customFocus,
              operationId: options?.operationId,
              shouldActivateOnComplete: options?.shouldActivateOnComplete,
            }).then(() => undefined)}
          onCancelCloneSummary={(operationId) => sessionTabs.cancelCloneSummary(operationId)}
          activeSessionTab={sessionTabs.activeTab}
          onCreateCloneGitBranch={sessionTabs.createCloneGitBranch}
        />
      ) : (
        <div style={{ flex: 1, minHeight: 0, overflow: "hidden" }}>
          <WelcomeView
            model={currentDefault.model}
            provider={currentDefault.provider}
            projects={projectsHook.projects}
            onAddProject={projectsHook.add}
            onSend={(...args) => void handleWelcomeSend(...args)}
            onModelChange={(m, p) => setWelcomeModel({ model: m, provider: p })}
            reasoningMode={reasoningMode}
            onReasoningModeChange={setReasoningMode}
          />
        </div>
      )}
    </div>
  ), [
    activeSession?.name, activeSessionId, currentDefault.model, currentDefault.provider, displayModel, displayProject?.path,
    displayProvider, displayReasoningMode, displaySession?.clone_parent_session_id, displaySession?.parent_session_id,
    displaySessionId,
    fileOperations, filePreview, fileTreeNav, forecastNav.panelMode, forecastNav.setPanelMode, forecastContent,
    fullscreenSwitching, handleAutoRename, handleCreateInProjectWithModel, handleCreateWithModel,
    handleOpenForecastDocs, handlePreviewFullscreenChange, handleSelectById, handleWelcomeSend,
    pendingFiles, pendingMessage, pendingSkills, pendingWorkingDir, projectsHook, refresh,
    sessionSummary, sessionTabs, setFileOperations, setPendingFiles, setPendingMessage, setPendingSkills, setPendingWorkingDir, setReasoningMode,
    setWelcomeModel, summaryGit, tabGit, terminal, terminalCwd, reasoningMode, updateModel,
  ]);

  return <><PanelSlot name="list">{list}</PanelSlot><PanelSlot name="detail">{detail}</PanelSlot>{tabGit.dialogs}</>;
});
