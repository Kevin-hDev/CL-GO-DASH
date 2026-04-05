import { useState, useEffect, useCallback, useRef } from "react";
import type { SessionMeta, SessionDetail } from "@/types/session";
import * as api from "@/services/sessions";
import { useFsEvent } from "./use-fs-event";
import { showToast } from "@/lib/toast-emitter";

type SubTab = "recent" | "archive" | "favorites";

export function useSessions() {
  const [recent, setRecent] = useState<SessionMeta[]>([]);
  const [archive, setArchive] = useState<SessionMeta[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [detail, setDetail] = useState<SessionDetail | null>(null);
  const [subTab, setSubTab] = useState<SubTab>("recent");
  const [loading, setLoading] = useState(false);
  const selectedRef = useRef<string | null>(null);

  // Keep ref in sync for use in callbacks
  selectedRef.current = selectedId;

  const loadList = useCallback(async () => {
    try {
      const [r, a] = await Promise.all([
        api.listSessions(30, 0),
        api.listSessions(30, 30),
      ]);
      setRecent(r);
      setArchive(a);
      if (r.length > 0 && !selectedRef.current) {
        setSelectedId(r[0].id);
      }
    } catch (e) {
      showToast("Failed to load sessions");
    }
  }, []);

  useEffect(() => { loadList(); }, [loadList]);

  const refreshDetail = useCallback(async () => {
    const id = selectedRef.current;
    if (!id) return;
    try {
      const d = await api.getSessionDetail(id);
      setDetail(d);
    } catch {
      // Silent — detail refresh is best-effort
    }
  }, []);

  // Auto-refresh list + detail when session files change
  const onSessionsChanged = useCallback(() => {
    loadList();
    refreshDetail();
  }, [loadList, refreshDetail]);

  useFsEvent("fs:sessions-changed", onSessionsChanged);

  const loadDetail = useCallback(async (id: string) => {
    setSelectedId(id);
    setLoading(true);
    try {
      const d = await api.getSessionDetail(id);
      setDetail(d);
    } catch (e) {
      showToast("Failed to load detail");
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
      showToast("Failed to rename");
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
      showToast("Failed to delete");
    }
  }, [selectedId]);

  const allSessions = [...recent, ...archive];
  const items = subTab === "recent" ? recent : subTab === "archive" ? archive : allSessions;

  return {
    items, recent, archive,
    selectedId, detail, loading,
    subTab, setSubTab,
    loadDetail, renameSession, deleteSession, cleanup,
  };

  async function cleanup() {
    try {
      const overflow = await api.listSessions(100, 60);
      for (const s of overflow) {
        await api.deleteSessionFile(s.file_path);
      }
    } catch {
      // Silent
    }
  }
}
