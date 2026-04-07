import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
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

  const create = useCallback(async (name: string, model: string) => {
    const session = await invoke<AgentSessionMeta>("create_agent_session", { name, model });
    await refresh();
    return session;
  }, [refresh]);

  const rename = useCallback(async (id: string, name: string) => {
    await invoke("rename_agent_session", { id, name });
    await refresh();
  }, [refresh]);

  const remove = useCallback(async (id: string) => {
    await invoke("delete_agent_session", { id });
    await refresh();
  }, [refresh]);

  return { sessions, loading, refresh, create, rename, remove };
}
