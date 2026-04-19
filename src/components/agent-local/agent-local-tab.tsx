import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { ConversationList } from "./conversation-list";
import { TabBar } from "./tab-bar";
import { ChatView } from "./chat-view";
import { WelcomeView } from "./welcome-view";
import { useAgentSessions } from "@/hooks/use-agent-sessions";
import { useAgentTabs } from "@/hooks/use-agent-tabs";
import { useProjects } from "@/hooks/use-projects";
import { useTerminal } from "@/hooks/use-terminal";

interface OllamaModel {
  name: string;
}

function useDefaultModel(): { model: string; provider: string } {
  const [state, setState] = useState({ model: "gemma4:e4b", provider: "ollama" });
  useEffect(() => {
    invoke<OllamaModel[]>("list_ollama_models")
      .then((models) => {
        if (models.length > 0) setState({ model: models[0].name, provider: "ollama" });
      })
      .catch((e) => console.warn("Ollama models:", e));
  }, []);
  return state;
}

export function AgentLocalTab(): { list: React.ReactNode; detail: React.ReactNode; onCreate: () => void; onShowWelcome: () => void } {
  const { t } = useTranslation();
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

  const currentDefault = welcomeModel ?? { model: defaultModel, provider: defaultProvider };
  const model = activeSession?.model ?? currentDefault.model;
  const provider = activeSession?.provider ?? currentDefault.provider;

  const handleCreate = useCallback(() => {
    tabState.deselectTab();
  }, [tabState]);

  const handleCreateWithModel = useCallback(
    async (newModel: string, newProvider: string) => {
      const name = t("agentLocal.newSession");
      const session = await create(name, newModel, newProvider);
      await tabState.addTab(session.id, session.name);
    },
    [create, tabState, t],
  );

  const [pendingMessage, setPendingMessage] = useState<string | null>(null);
  const [pendingWorkingDir, setPendingWorkingDir] = useState<string | undefined>(undefined);
  const [pendingSkills, setPendingSkills] = useState<{ name: string; content: string }[] | undefined>(undefined);

  const handleWelcomeSend = useCallback(
    async (text: string, projectId?: string, skills?: { name: string; content: string }[]) => {
      const name = text.slice(0, 40).trim() || t("agentLocal.newSession");
      const m = welcomeModel ?? { model: defaultModel, provider: defaultProvider };
      const project = projectId ? projectsHook.projects.find((p) => p.id === projectId) : undefined;
      const session = await create(name, m.model, m.provider, projectId);
      setPendingMessage(text);
      setPendingWorkingDir(project?.path);
      setPendingSkills(skills);
      setWelcomeModel(null);
      await tabState.addTab(session.id, session.name);
    },
    [create, tabState, defaultModel, defaultProvider, welcomeModel, t, projectsHook.projects],
  );

  const handleCreateInProject = useCallback(
    async (projectId: string) => {
      const name = t("agentLocal.newSession");
      const session = await create(name, defaultModel, defaultProvider, projectId);
      await tabState.addTab(session.id, session.name);
    },
    [create, tabState, defaultModel, defaultProvider, t],
  );

  const handleSelect = useCallback(async (id: string) => {
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
      onSelect={handleSelect}
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
            initialMessage={pendingMessage ?? undefined}
            initialWorkingDir={pendingWorkingDir}
            initialSkills={pendingSkills}
            onInitialMessageSent={() => { setPendingMessage(null); setPendingWorkingDir(undefined); setPendingSkills(undefined); }}
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
          />
        </div>
      )}
    </div>
  );

  return { list, detail, onCreate: handleCreate, onShowWelcome: tabState.deselectTab };
}
