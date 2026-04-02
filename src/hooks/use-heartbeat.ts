import { useState, useEffect, useCallback } from "react";
import type { ScheduledWakeup } from "@/types/config";
import { invoke } from "@tauri-apps/api/core";
import * as api from "@/services/heartbeat";

export function useHeartbeat() {
  const [wakeups, setWakeups] = useState<ScheduledWakeup[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [hbActive, setHbActive] = useState(false);

  const load = useCallback(async () => {
    try {
      const [list, hb] = await Promise.all([
        api.listWakeups(),
        api.getHeartbeatConfig(),
      ]);
      setWakeups(list);
      setHbActive(hb.active);
      if (list.length > 0 && !selectedId) {
        setSelectedId(list[0].id);
      }
    } catch (e) {
      console.error("Failed to load heartbeat:", e);
    }
  }, [selectedId]);

  useEffect(() => {
    load();
  }, [load]);

  const addWakeup = useCallback(async () => {
    try {
      const w = await api.createWakeup({ time: "08:00", mode: "auto" });
      setWakeups((prev) => [...prev, w]);
      setSelectedId(w.id);
    } catch (e) {
      console.error("Failed to create:", e);
    }
  }, []);

  const saveWakeup = useCallback(async (w: ScheduledWakeup) => {
    try {
      await api.updateWakeup(w);
      setWakeups((prev) => prev.map((x) => (x.id === w.id ? w : x)));
    } catch (e) {
      console.error("Failed to save:", e);
    }
  }, []);

  const removeWakeup = useCallback(async (id: string) => {
    try {
      await api.deleteWakeup(id);
      setWakeups((prev) => prev.filter((w) => w.id !== id));
      setSelectedId(null);
    } catch (e) {
      console.error("Failed to delete:", e);
    }
  }, []);

  const toggleHeartbeat = useCallback(async (active: boolean) => {
    try {
      await api.setHeartbeatActive(active);
      setHbActive(active);
    } catch (e) {
      console.error("Failed to toggle:", e);
    }
  }, []);

  const selected = wakeups.find((w) => w.id === selectedId) ?? null;

  return {
    wakeups,
    selected,
    selectedId,
    setSelectedId,
    hbActive,
    addWakeup,
    saveWakeup,
    removeWakeup,
    toggleHeartbeat,
    runWakeup,
  };

  async function runWakeup(id: string) {
    try {
      await invoke("run_wakeup", { id });
    } catch (e) {
      console.error("Failed to run:", e);
    }
  }
}
