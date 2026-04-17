import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { AgentSessionMeta } from "@/types/agent";

export function useAgentSessions() {
  const [sessions, setSessions] = useState<AgentSessionMeta[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const list = await invoke<AgentSessionMeta[]>("list_agent_sessions");
      setSessions(list);
    } catch {
      setSessions([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useEffect(() => {
    const unlisten = listen("wakeup-completed", () => {
      refresh();
    });
    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [refresh]);

  const create = useCallback(
    async (name: string, model: string, provider: string = "ollama", projectId?: string) => {
      const session = await invoke<AgentSessionMeta>("create_agent_session", {
        name,
        model,
        provider,
        projectId: projectId ?? null,
      });
      await refresh();
      return session;
    },
    [refresh],
  );

  const rename = useCallback(async (id: string, name: string) => {
    await invoke("rename_agent_session", { id, name });
    await refresh();
  }, [refresh]);

  const remove = useCallback(async (id: string) => {
    await invoke("delete_agent_session", { id });
    await refresh();
  }, [refresh]);

  const updateModel = useCallback(
    async (id: string, model: string, provider: string = "ollama") => {
      const session = await invoke<Record<string, unknown>>("get_agent_session", { id });
      session.model = model;
      session.provider = provider;
      await invoke("save_agent_session", { session });
      await refresh();
    },
    [refresh],
  );

  return { sessions, loading, refresh, create, rename, remove, updateModel };
}
