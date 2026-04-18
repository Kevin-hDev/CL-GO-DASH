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
      .catch(() => {});
  }, []);
  return state;
}

export function AgentLocalTab(): { list: React.ReactNode; detail: React.ReactNode; onCreate: () => void; onShowWelcome: () => void } {
  const { t } = useTranslation();
  const { sessions, refresh, create, rename, remove, updateModel } = useAgentSessions();
  const tabState = useAgentTabs();
  const projectsHook = useProjects();
  const { model: defaultModel, provider: defaultProvider } = useDefaultModel();

  const activeSession = tabState.activeSessionId
    ? sessions.find((s) => s.id.localeCompare(tabState.activeSessionId!) === 0)
    : null;
  const model = activeSession?.model ?? defaultModel;
  const provider = activeSession?.provider ?? defaultProvider;

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

  const handleWelcomeSend = useCallback(
    async (text: string, projectId?: string) => {
      const name = text.slice(0, 40).trim() || t("agentLocal.newSession");
      const session = await create(name, defaultModel, defaultProvider, projectId);
      setPendingMessage(text);
      await tabState.addTab(session.id, session.name);
    },
    [create, tabState, defaultModel, defaultProvider, t],
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
      onDelete={remove}
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
            onSelect={tabState.selectTab}
            onClose={tabState.closeTab}
            onAdd={handleCreate}
            onRename={tabState.renameTab}
            onReorder={tabState.reorderTabs}
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
            onInitialMessageSent={() => setPendingMessage(null)}
          />
        </div>
      ) : (
        <div style={{ flex: 1, minHeight: 0, overflow: "hidden" }}>
          <WelcomeView
            model={defaultModel}
            provider={defaultProvider}
            projects={projectsHook.projects}
            onAddProject={projectsHook.add}
            onSend={handleWelcomeSend}
            onModelChange={() => {}}
          />
        </div>
      )}
    </div>
  );

  return { list, detail, onCreate: handleCreate, onShowWelcome: tabState.deselectTab };
}
