import { useState, useCallback, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { EmptyState } from "@/components/ui/empty-state";
import { ConversationList } from "./conversation-list";
import { TabBar } from "./tab-bar";
import { ChatView } from "./chat-view";
import { useAgentSessions } from "@/hooks/use-agent-sessions";
import { useAgentTabs } from "@/hooks/use-agent-tabs";

interface OllamaModel {
  name: string;
}

function useDefaultModel(): string {
  const [defaultModel, setDefaultModel] = useState("gemma4:e4b");
  useEffect(() => {
    invoke<OllamaModel[]>("list_ollama_models")
      .then((models) => {
        if (models.length > 0) setDefaultModel(models[0].name);
      })
      .catch(() => {});
  }, []);
  return defaultModel;
}

export function AgentLocalTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const { sessions, create, rename, remove, updateModel } = useAgentSessions();
  const tabState = useAgentTabs();
  const defaultModel = useDefaultModel();

  const activeSession = tabState.activeSessionId
    ? sessions.find((s) => s.id.localeCompare(tabState.activeSessionId!) === 0)
    : null;
  const model = activeSession?.model ?? defaultModel;

  const handleCreate = useCallback(async () => {
    const name = t("agentLocal.newSession");
    const session = await create(name, defaultModel);
    await tabState.addTab(session.id, session.name);
  }, [create, tabState, defaultModel, t]);

  const handleSelect = useCallback(async (id: string) => {
    const idx = tabState.tabs.findIndex((tab) => tab.session_id.localeCompare(id) === 0);
    if (idx >= 0) {
      await tabState.selectTab(idx);
    } else {
      const s = sessions.find((s) => s.id.localeCompare(id) === 0);
      await tabState.addTab(id, s?.name ?? "Chat");
    }
  }, [tabState, sessions]);

  const list = (
    <ConversationList
      sessions={sessions}
      selectedId={tabState.activeSessionId}
      onSelect={handleSelect}
      onCreate={handleCreate}
      onRename={rename}
      onDelete={remove}
    />
  );

  const detail = (
    <div style={{ display: "flex", flexDirection: "column", height: "100%", overflow: "hidden" }}>
      {tabState.tabs.length > 0 && (
        <div style={{ flexShrink: 0 }}>
          <TabBar
            tabs={tabState.tabs}
            activeIndex={tabState.activeIndex}
            onSelect={tabState.selectTab}
            onClose={tabState.closeTab}
            onAdd={handleCreate}
            onRename={tabState.renameTab}
          />
        </div>
      )}
      {tabState.activeSessionId ? (
        <div style={{ flex: 1, minHeight: 0, overflow: "hidden" }}>
          <ChatView
            sessionId={tabState.activeSessionId}
            model={model}
            onModelChange={(m) => {
              if (tabState.activeSessionId && updateModel) {
                updateModel(tabState.activeSessionId, m);
              }
            }}
          />
        </div>
      ) : (
        <div style={{ flex: 1, display: "flex", alignItems: "center", justifyContent: "center" }}>
          <EmptyState message={t("agentLocal.selectOrCreate")} />
        </div>
      )}
    </div>
  );

  return { list, detail };
}
