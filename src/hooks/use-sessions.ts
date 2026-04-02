import { useState, useEffect, useCallback } from "react";
import type { SessionMeta, SessionDetail } from "@/types/session";
import * as api from "@/services/sessions";
import { useFsEvent } from "./use-fs-event";

type SubTab = "recent" | "archive";

export function useSessions() {
  const [recent, setRecent] = useState<SessionMeta[]>([]);
  const [archive, setArchive] = useState<SessionMeta[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [subTab, setSubTab] = useState<SubTab>("recent");
  const [loading, setLoading] = useState(false);

  const loadList = useCallback(async () => {
    try {
      const [r, a] = await Promise.all([
        api.listSessions(30, 0),
        api.listSessions(30, 30),
      ]);
      setRecent(r);
      setArchive(a);
      if (r.length > 0 && !selectedId) {
        setSelectedId(r[0].id);
      }
    } catch (e) {
      console.error("Failed to load sessions:", e);
    }
  }, [selectedId]);

  useEffect(() => { loadList(); }, [loadList]);
  useFsEvent("fs:sessions-changed", loadList);

  const loadDetail = useCallback(async (id: string) => {
    setSelectedId(id);
    setLoading(true);
    try {
      const d = await api.getSessionDetail(id);
      setDetail(d);
    } catch (e) {
      console.error("Failed to load detail:", e);
      setDetail(null);
    } finally {
      setLoading(false);
    }
  }, []);

  const renameSession = useCallback(async (id: string, name: string) => {
    try {
      await api.renameSession(id, name);
      const update = (list: SessionMeta[]) =>
        list.map((s) => (s.id === id ? { ...s, custom_name: name } : s));
      setRecent(update);
      setArchive(update);
    } catch (e) {
      console.error("Failed to rename:", e);
    }
  }, []);

  const deleteSession = useCallback(async (filePath: string, id: string) => {
    try {
      await api.deleteSessionFile(filePath);
      setRecent((prev) => prev.filter((s) => s.id !== id));
      setArchive((prev) => prev.filter((s) => s.id !== id));
      if (selectedId === id) {
        setSelectedId(null);
        setDetail(null);
      }
    } catch (e) {
      console.error("Failed to delete:", e);
    }
  }, [selectedId]);

  const items = subTab === "recent" ? recent : archive;

  return {
    items, recent, archive,
    selectedId, detail, loading,
    subTab, setSubTab,
    loadDetail, renameSession, deleteSession, cleanup,
  };

  async function cleanup() {
    // Delete sessions beyond 60 (archive overflow)
    try {
      const overflow = await api.listSessions(100, 60);
      for (const s of overflow) {
        await api.deleteSessionFile(s.file_path);
      }
    } catch {
      // Silent — cleanup is best-effort
    }
  }
}
