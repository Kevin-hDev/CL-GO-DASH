import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { EmptyState } from "@/components/ui/empty-state";
import { ConversationList } from "./conversation-list";
import { TabBar } from "./tab-bar";
import { ChatView } from "./chat-view";
import { ContextBar } from "./context-bar";
import { useAgentSessions } from "@/hooks/use-agent-sessions";
import { useAgentTabs } from "@/hooks/use-agent-tabs";
import { useContextBar } from "@/hooks/use-context-bar";

export function AgentLocalTab(): { list: React.ReactNode; detail: React.ReactNode } {
  const { t } = useTranslation();
  const { sessions, create, rename, remove } = useAgentSessions();
  const tabState = useAgentTabs();

  const activeSession = tabState.activeSessionId
    ? sessions.find((s) => s.id.localeCompare(tabState.activeSessionId!) === 0)
    : null;
  const model = activeSession?.model ?? "qwen3.5:latest";
  const contextBar = useContextBar(model, 0);

  const handleCreate = useCallback(async () => {
    const session = await create("Nouvelle conversation", "qwen3.5:latest");
    await tabState.addTab(session.id, session.name);
  }, [create, tabState]);

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
    <div style={{ display: "flex", flexDirection: "column", height: "100%" }}>
      {tabState.tabs.length > 0 && (
        <>
          <TabBar
            tabs={tabState.tabs}
            activeIndex={tabState.activeIndex}
            onSelect={tabState.selectTab}
            onClose={tabState.closeTab}
            onAdd={handleCreate}
            onRename={tabState.renameTab}
          />
          <ContextBar percentage={contextBar.percentage} color={contextBar.color} />
        </>
      )}
      {tabState.activeSessionId ? (
        <ChatView sessionId={tabState.activeSessionId} model={model} />
      ) : (
        <div style={{
          flex: 1, display: "flex", alignItems: "center", justifyContent: "center",
        }}>
          <EmptyState message={t("agentLocal.selectOrCreate")} />
        </div>
      )}
    </div>
  );

  return { list, detail };
}
