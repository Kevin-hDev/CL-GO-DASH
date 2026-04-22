import { useState, useCallback, useEffect, useRef } from "react";
import { ConversationList } from "./conversation-list";
import { TabBar } from "./tab-bar";
import { ChatView } from "./chat-view";
import { WelcomeView } from "./welcome-view";
import { useAgentSessions } from "@/hooks/use-agent-sessions";
import { useAgentTabs } from "@/hooks/use-agent-tabs";
import { useProjects } from "@/hooks/use-projects";
import { useTerminal } from "@/hooks/use-terminal";
import { useDefaultModel } from "@/hooks/use-default-model";
import { useSessionActions } from "@/hooks/use-session-actions";

interface AgentLocalTabProps {
  requestedSessionId?: string | null;
  onSessionChange?: (id: string | null) => void;
}

export function AgentLocalTab(props?: AgentLocalTabProps): { list: React.ReactNode; detail: React.ReactNode; onCreate: () => void; onShowWelcome: () => void } {
  const { sessions, refresh, create, rename, remove, updateModel } = useAgentSessions();
  const tabState = useAgentTabs();
  const projectsHook = useProjects();

  const activeSession = tabState.activeSessionId
    ? sessions.find((s) => s.id.localeCompare(tabState.activeSessionId!) === 0)
    : null;
  const activeProject = activeSession?.project_id
    ? projectsHook.projects.find((p) => p.id === activeSession.project_id)
    : null;
  const terminalCwd = activeProject?.path || "";
  const terminal = useTerminal(terminalCwd);
  const { model: defaultModel, provider: defaultProvider } = useDefaultModel();
  const [welcomeModel, setWelcomeModel] = useState<{ model: string; provider: string } | null>(null);
  const [thinking, setThinking] = useState(false);

  const currentDefault = welcomeModel ?? { model: defaultModel, provider: defaultProvider };
  const model = activeSession?.model ?? currentDefault.model;
  const provider = activeSession?.provider ?? currentDefault.provider;

  const sessionActions = useSessionActions({ create, tabState, rename, defaultModel, defaultProvider, welcomeModel, setWelcomeModel, projectsHook });
  const { pendingMessage, setPendingMessage, pendingWorkingDir, setPendingWorkingDir, pendingSkills, setPendingSkills, pendingFiles, setPendingFiles, handleCreate, handleCreateWithModel, handleWelcomeSend, handleAutoRename, handleCreateInProject } = sessionActions;

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      const onMac = navigator.userAgent.includes("Mac");
      const toggle = onMac ? (e.metaKey && e.code === "KeyJ") : (e.ctrlKey && e.code === "KeyJ");
      if (!toggle) return;
      if (!tabState.activeSessionId) return;
      const target = e.target as HTMLElement;
      const isEditableTarget =
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement ||
        target.isContentEditable;
      if (isEditableTarget) return;
      e.preventDefault();
      if (!terminal.isOpen && terminal.tabs.length === 0) {
        terminal.addTab(terminalCwd);
      } else {
        terminal.togglePanel();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [tabState.activeSessionId, terminal, terminalCwd]);

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
      onDeleteProject={projectsHook.remove}
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
            onSelect={tabState.selectTab}
            onClose={tabState.closeTab}
            onAdd={handleCreate}
            onRename={tabState.renameTab}
            onReorder={tabState.reorderTabs}
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
        <div style={{ flex: 1, minHeight: 0, overflow: "hidden" }}>
          <ChatView
            sessionId={tabState.activeSessionId}
            model={model}
            provider={provider}
            projects={projectsHook.projects}
            onAddProject={projectsHook.add}
            onSessionsRefresh={refresh}
            onApplySwitch={(m, p) => {
              if (tabState.activeSessionId && updateModel) {
                updateModel(tabState.activeSessionId, m, p);
              }
            }}
            onNewSession={handleCreateWithModel}
            onAutoRename={handleAutoRename}
            initialMessage={pendingMessage ?? undefined}
            initialWorkingDir={pendingWorkingDir}
            initialSkills={pendingSkills}
            initialFiles={pendingFiles}
            thinking={thinking}
            onToggleThinking={() => setThinking((v) => !v)}
            onInitialMessageSent={() => { setPendingMessage(null); setPendingWorkingDir(undefined); setPendingSkills(undefined); setPendingFiles(undefined); }}
            terminalState={terminal}
          />
        </div>
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

  return { list, detail, onCreate: handleCreate, onShowWelcome: tabState.deselectTab };
}
