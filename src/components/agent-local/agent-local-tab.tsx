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
  const terminalCwd = activeProject?.path || "";
  const fileTree = useFileTree(activeSessionId, activeProject?.path);
  const forecast = useForecastPanel(activeSessionId ?? null);
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
          onToggleTerminal={() => {
            if (!terminal.isOpen && terminal.tabs.length === 0) {
              terminal.addTab(terminalCwd);
            } else {
              terminal.togglePanel();
            }
          }}
        />
      </div>
      {activeSessionId ? (
        <AgentChatDetail
          sessionId={activeSessionId}
          model={model}
          provider={provider}
          projects={projectsHook.projects}
          activeProjectPath={activeProject?.path}
          pendingMessage={pendingMessage}
          pendingWorkingDir={pendingWorkingDir}
          pendingSkills={pendingSkills}
          pendingFiles={pendingFiles}
          reasoningMode={reasoningMode}
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
          parentSessionId={activeSession?.parent_session_id}
          onOpenSubagent={(id) => void handleSelectById(id)}
          onGoToParent={() => {
            if (activeSession?.parent_session_id) {
              void handleSelectById(activeSession.parent_session_id);
            }
          }}
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
    activeProject?.path, activeSession?.name, activeSession?.parent_session_id, activeSessionId, currentDefault.model, currentDefault.provider,
    fileOperations, filePreview, fileTreeNav, forecastNav.panelMode, forecastNav.setPanelMode, forecastContent,
    fullscreenSwitching, handleAutoRename, handleCreate, handleCreateInProjectWithModel, handleCreateWithModel,
    handleOpenForecastDocs, handlePreviewFullscreenChange, handleSelectById, handleWelcomeSend, model,
    pendingFiles, pendingMessage, pendingSkills, pendingWorkingDir, projectsHook, provider, refresh,
    setFileOperations, setPendingFiles, setPendingMessage, setPendingSkills, setPendingWorkingDir, setReasoningMode,
    setWelcomeModel, terminal, terminalCwd, reasoningMode, updateModel,
  ]);

  return <><PanelSlot name="list">{list}</PanelSlot><PanelSlot name="detail">{detail}</PanelSlot></>;
});
