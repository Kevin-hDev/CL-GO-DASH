import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState, memo } from "react";
import { useTranslation } from "react-i18next";
import { ConversationList } from "./conversation-list";
import { TabBar } from "./tab-bar";
import { AgentChatDetail } from "./agent-chat-detail";
import { WelcomeView } from "./welcome-view";
import { useAgentLocalTab } from "@/hooks/use-agent-local-tab";
import { useFileTree } from "@/hooks/use-file-tree";
import { useForecastPanel } from "@/hooks/use-forecast-panel";
import { useAgentLocalPanelNav } from "@/hooks/use-agent-local-panel-nav";
import { ForecastPanel } from "@/components/forecast/forecast-panel";
import { openForecastDocsWindow } from "@/components/forecast/open-forecast-docs";
import type { AgentLocalTabProps } from "./agent-local-tab-types";
import "./agent-local-tab.css";

export const AgentLocalTab = memo(function AgentLocalTab({
  navState,
  onSessionChange,
  onNavChange,
  onNavReplace,
  listFocused = true,
  reportContent,
}: AgentLocalTabProps) {
  const { t } = useTranslation();
  const s = useAgentLocalTab({ navState, onSessionChange, onNavChange, onNavReplace, listFocused });
  const { sessions, refresh, rename, remove, updateModel } = s;
  const { tabState, projectsHook, terminal, activeSession } = s;
  const { model, provider, currentDefault, activeProject } = s;
  const { filePreview, fileOperations, setFileOperations } = s;
  const { thinking, setThinking, setWelcomeModel } = s;
  const { sessionActions, handleSelectById, handleDeleteProject } = s;
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
  const fileTree = useFileTree(tabState.activeSessionId, activeProject?.path);
  const forecast = useForecastPanel(tabState.activeSessionId ?? null);
  useAgentLocalPanelNav({ navState, fileTree, forecast, onNavChange, onNavReplace });
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
      activeSection={forecast.activeSection}
      navOpen={forecast.navOpen}
      currentAnalysisId={forecast.currentAnalysisId}
      fullscreen={filePreview.fullscreen}
      onSectionChange={forecast.setSection}
      onToggleNav={forecast.toggleNav}
      onLoadAnalysis={forecast.loadAnalysis}
      onFocusAnalysis={forecast.focusAnalysis}
      onPanelExtraWidthChange={filePreview.setExtraWidth}
      onCloseAnalysis={forecast.closeAnalysis}
      onFullscreenChange={handlePreviewFullscreenChange}
    />
  ), [filePreview.fullscreen, filePreview.setExtraWidth, forecast, handlePreviewFullscreenChange]);

  const list = useMemo(() => (
    <ConversationList
      sessions={sessions}
      projects={projectsHook.projects}
      selectedId={tabState.activeSessionId}
      onSelect={(id) => void handleSelectById(id)}
      onCreate={handleCreate}
      onRename={(id, name) => void rename(id, name)}
      onDelete={(id) => void tabState.closeBySessionId(id).then(() => remove(id))}
      onNewSessionInProject={(pid) => void handleCreateInProject(pid)}
      onRenameProject={(id, name) => void projectsHook.rename(id, name)}
      onDeleteProject={handleDeleteProject}
      onOpenFolder={(path) => void projectsHook.openFolder(path)}
      onReorderProjects={(ids) => void projectsHook.reorder(ids)}
    />
  ), [handleCreate, handleCreateInProject, handleDeleteProject, handleSelectById, projectsHook, remove, rename, sessions, tabState]);

  const detail = useMemo(() => (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
      {tabState.tabs.length > 0 && (
        <div style={{ flexShrink: 0 }}>
          <TabBar
            tabs={tabState.tabs}
            activeIndex={tabState.activeIndex}
            canAddTab={tabState.canAddTab}
            sessionId={tabState.activeSessionId ?? null}
            terminalOpen={terminal.isOpen}
            previewOpen={filePreview.open}
            panelMode={forecast.panelMode}
            showForecastDocs={filePreview.open && forecast.panelMode === "forecast"}
            onSelect={(i) => void tabState.selectTab(i)}
            onClose={(i) => void tabState.closeTab(i)}
            onAdd={handleCreate}
            onRename={(i, name) => void tabState.renameTab(i, name)}
            onReorder={(from, to) => void tabState.reorderTabs(from, to)}
            onTogglePreview={filePreview.toggleOpen}
            onOpenForecastDocs={handleOpenForecastDocs}
            onPanelModeChange={forecast.setPanelMode}
            onToggleTerminal={() => {
              if (!terminal.isOpen && terminal.tabs.length === 0) {
                terminal.addTab(terminalCwd);
              } else {
                terminal.togglePanel();
              }
            }}
          />
        </div>
      )}
      {tabState.activeSessionId ? (
        <AgentChatDetail
          sessionId={tabState.activeSessionId}
          model={model}
          provider={provider}
          projects={projectsHook.projects}
          activeProjectPath={activeProject?.path}
          pendingMessage={pendingMessage}
          pendingWorkingDir={pendingWorkingDir}
          pendingSkills={pendingSkills}
          pendingFiles={pendingFiles}
          thinking={thinking}
          terminal={terminal}
          filePreview={filePreview}
          fullscreenSwitching={fullscreenSwitching}
          fileOperations={fileOperations}
          fileTree={fileTree}
          onAddProject={projectsHook.add}
          onSessionsRefresh={() => void refresh()}
          onUpdateModel={(id, m, p) => void updateModel(id, m, p)}
          onNewSession={(m, p) => void handleCreateWithModel(m, p)}
          onNewSessionInProject={(m, p, pid) =>
            void handleCreateInProjectWithModel(m, p, pid)}
          onAutoRename={(id, msg) => void handleAutoRename(id, msg)}
          onToggleThinking={() => setThinking((v) => !v)}
          onInitialMessageSent={() => {
            setPendingMessage(null);
            setPendingWorkingDir(undefined);
            setPendingSkills(undefined);
            setPendingFiles(undefined);
          }}
          onFileOperationsChange={setFileOperations}
          onPreviewFullscreenChange={handlePreviewFullscreenChange}
          panelMode={forecast.panelMode}
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
            thinking={thinking}
            onToggleThinking={() => setThinking((v) => !v)}
          />
        </div>
      )}
    </div>
  ), [
    activeProject?.path, activeSession?.parent_session_id, currentDefault.model, currentDefault.provider,
    fileOperations, filePreview, fileTree, forecast.panelMode, forecast.setPanelMode, forecastContent,
    fullscreenSwitching, handleAutoRename, handleCreate, handleCreateInProjectWithModel, handleCreateWithModel,
    handleOpenForecastDocs, handlePreviewFullscreenChange, handleSelectById, handleWelcomeSend, model,
    pendingFiles, pendingMessage, pendingSkills, pendingWorkingDir, projectsHook, provider, refresh,
    setFileOperations, setPendingFiles, setPendingMessage, setPendingSkills, setPendingWorkingDir, setThinking,
    setWelcomeModel, tabState, terminal, terminalCwd, thinking, updateModel,
  ]);

  useLayoutEffect(() => { reportContent({ list, detail }); }, [reportContent, list, detail]);

  return null;
});
