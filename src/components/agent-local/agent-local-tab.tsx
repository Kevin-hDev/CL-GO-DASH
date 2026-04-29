import { useState, useCallback, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ConversationList } from "./conversation-list";
import { TabBar } from "./tab-bar";
import { AgentChatDetail } from "./agent-chat-detail";
import { WelcomeView } from "./welcome-view";
import { useAgentSessions } from "@/hooks/use-agent-sessions";
import { useAgentTabs } from "@/hooks/use-agent-tabs";
import { useProjects } from "@/hooks/use-projects";
import { useTerminal } from "@/hooks/use-terminal";
import { useDefaultModel } from "@/hooks/use-default-model";
import { useAvailableModels } from "@/hooks/use-available-models";
import { useSessionActions } from "@/hooks/use-session-actions";
import { useFilePreview } from "@/hooks/use-file-preview";
import { useAgentLocalShortcuts } from "@/hooks/use-agent-local-shortcuts";
import type { FileOperation } from "@/types/file-preview";
import type { AgentLocalTabProps } from "./agent-local-tab-types";
import "./agent-local-tab.css";

export function AgentLocalTab(props?: AgentLocalTabProps): { list: React.ReactNode; detail: React.ReactNode; onCreate: () => void; onShowWelcome: () => void; previewOpen: boolean; onTogglePreview: () => void } {
  const { sessions, refresh, create, rename, remove, updateModel } = useAgentSessions();
  const tabState = useAgentTabs();
  const projectsHook = useProjects();

  const activeSession = tabState.activeSessionId
    ? sessions.find((s) => s.id.localeCompare(tabState.activeSessionId!) === 0)
    : null;
  const activeProject = activeSession?.project_id
    ? projectsHook.projects.find((p) => p.id === activeSession.project_id)
    : null;
  const terminalGroupKey = activeProject?.id || "__default__";
  const terminalCwd = activeProject?.path || "";
  const validGroupKeys = projectsHook.projects.map((p) => p.id);
  const terminal = useTerminal(terminalGroupKey, terminalCwd, validGroupKeys);
  const { model: defaultModel, provider: defaultProvider } = useDefaultModel();
  const [welcomeModel, setWelcomeModel] = useState<{ model: string; provider: string } | null>(null);
  const [thinking, setThinking] = useState(false);
  const [fileOperations, setFileOperations] = useState<FileOperation[]>([]);

  const { groups: availableModels } = useAvailableModels();

  const currentDefault = welcomeModel ?? { model: defaultModel, provider: defaultProvider };
  const model = activeSession?.model ?? currentDefault.model;
  const provider = activeSession?.provider ?? currentDefault.provider;
  const filePreview = useFilePreview(tabState.activeSessionId ?? null, fileOperations);

  useEffect(() => {
    if (!model || availableModels.size === 0) return;
    const providerModels = availableModels.get(provider);
    const stillExists = providerModels?.some((m) => m.id === model);
    if (stillExists) return;
    const allModels = Array.from(availableModels.values()).flat();
    if (allModels.length === 0) return;
    const fallback = allModels[0];
    if (tabState.activeSessionId) {
      updateModel(tabState.activeSessionId, fallback.id, fallback.provider_id);
    } else {
      setWelcomeModel({ model: fallback.id, provider: fallback.provider_id });
    }
  }, [availableModels, model, provider, tabState.activeSessionId, updateModel]);

  const sessionActions = useSessionActions({ create, tabState, rename, defaultModel, defaultProvider, welcomeModel, setWelcomeModel, projectsHook });
  const { pendingMessage, setPendingMessage, pendingWorkingDir, setPendingWorkingDir, pendingSkills, setPendingSkills, pendingFiles, setPendingFiles, handleCreate, handleCreateWithModel, handleWelcomeSend, handleAutoRename, handleCreateInProject } = sessionActions;

  useAgentLocalShortcuts({
    activeSessionId: tabState.activeSessionId,
    terminalOpen: terminal.isOpen,
    terminalTabsCount: terminal.tabs.length,
    terminalCwd,
    onAddTerminalTab: terminal.addTab,
    onToggleTerminal: terminal.togglePanel,
  });

  const prevSessionRef = useRef(tabState.activeSessionId);
  useEffect(() => {
    if (tabState.activeSessionId !== prevSessionRef.current) {
      prevSessionRef.current = tabState.activeSessionId;
      props?.onSessionChange?.(tabState.activeSessionId ?? null);
    }
  }, [tabState.activeSessionId, props?.onSessionChange]);

  useEffect(() => {
    if (props?.requestedSessionId === undefined) return;
    const requested = props.requestedSessionId;
    if (requested === tabState.activeSessionId) return;
    if (requested === null) {
      tabState.deselectTab();
    } else {
      handleSelectById(requested);
    }
  }, [props?.requestedSessionId]);

  const handleSelectById = useCallback(async (id: string) => {
    const existingIdx = tabState.tabs.findIndex((tab) => tab.session_id.localeCompare(id) === 0);
    if (existingIdx >= 0) {
      await tabState.selectTab(existingIdx);
      return;
    }
    const label = sessions.find((s) => s.id.localeCompare(id) === 0)?.name ?? "Chat";
    if (tabState.activeIndex >= 0 && tabState.activeIndex < tabState.tabs.length) {
      await tabState.updateTab(tabState.activeIndex, id, label);
    } else {
      await tabState.addTab(id, label);
    }
  }, [tabState, sessions]);

  const list = (
    <ConversationList
      sessions={sessions}
      projects={projectsHook.projects}
      selectedId={tabState.activeSessionId}
      onSelect={handleSelectById}
      onCreate={handleCreate}
      onRename={rename}
      onDelete={async (id: string) => { await tabState.closeBySessionId(id); await remove(id); }}
      onNewSessionInProject={handleCreateInProject}
      onRenameProject={projectsHook.rename}
      onDeleteProject={async (projectId: string) => {
        const ptyIds = terminal.getGroupPtyIds(projectId);
        for (const id of ptyIds) {
          invoke("pty_kill", { id }).catch(() => {});
        }
        terminal.removeGroup(projectId);
        await projectsHook.remove(projectId);
      }}
      onOpenFolder={projectsHook.openFolder}
      onReorderProjects={projectsHook.reorder}
    />
  );

  const detail = (
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

            onSelect={tabState.selectTab}
            onClose={tabState.closeTab}
            onAdd={handleCreate}
            onRename={tabState.renameTab}
            onReorder={tabState.reorderTabs}
            onTogglePreview={filePreview.toggleOpen}
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
          fileOperations={fileOperations}
          onAddProject={projectsHook.add}
          onSessionsRefresh={refresh}
          onUpdateModel={updateModel}
          onNewSession={handleCreateWithModel}
          onAutoRename={handleAutoRename}
          onToggleThinking={() => setThinking((v) => !v)}
          onInitialMessageSent={() => {
            setPendingMessage(null);
            setPendingWorkingDir(undefined);
            setPendingSkills(undefined);
            setPendingFiles(undefined);
          }}
          onFileOperationsChange={setFileOperations}
        />
      ) : (
        <div style={{ flex: 1, minHeight: 0, overflow: "hidden" }}>
          <WelcomeView
            model={currentDefault.model}
            provider={currentDefault.provider}
            projects={projectsHook.projects}
            onAddProject={projectsHook.add}
            onSend={handleWelcomeSend}
            onModelChange={(m, p) => setWelcomeModel({ model: m, provider: p })}
            thinking={thinking}
            onToggleThinking={() => setThinking((v) => !v)}
          />
        </div>
      )}
    </div>
  );

  return {
    list,
    detail,
    onCreate: handleCreate,
    onShowWelcome: tabState.deselectTab,
    previewOpen: filePreview.open,
    onTogglePreview: filePreview.toggleOpen,
  };
}
