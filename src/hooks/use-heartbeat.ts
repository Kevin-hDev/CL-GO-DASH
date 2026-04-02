import { useState, useEffect, useCallback, useRef } from "react";
import type { ScheduledWakeup } from "@/types/config";
import { invoke } from "@tauri-apps/api/core";
import * as api from "@/services/heartbeat";

export function useHeartbeat() {
  const [wakeups, setWakeups] = useState<ScheduledWakeup[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [hbActive, setHbActive] = useState(false);
  const [stopAt, setStopAtState] = useState<string | null>(null);

  const load = useCallback(async () => {
    try {
      const [list, hb] = await Promise.all([
        api.listWakeups(),
        api.getHeartbeatConfig(),
      ]);
      setWakeups(list);
      setHbActive(hb.active);
      setStopAtState(hb.stop_at);
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
      // Default: demain 08h00
      const tomorrow = new Date();
      tomorrow.setDate(tomorrow.getDate() + 1);
      tomorrow.setHours(8, 0, 0, 0);
      const defaultTime = tomorrow.toISOString().slice(0, 16);
      const w = await api.createWakeup({ time: defaultTime, mode: "auto" });
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

  // Check stop_at every 30s — disable heartbeat when time is reached
  const stopAtRef = useRef(stopAt);
  stopAtRef.current = stopAt;
  const hbActiveRef = useRef(hbActive);
  hbActiveRef.current = hbActive;

  useEffect(() => {
    const interval = setInterval(async () => {
      const sa = stopAtRef.current;
      if (!sa || !hbActiveRef.current) return;

      const stopTime = new Date(sa).getTime();
      const now = Date.now();
      if (now >= stopTime) {
        try {
          await api.setHeartbeatActive(false);
          setHbActive(false);
          await api.setStopAt(null);
          setStopAtState(null);
        } catch (e) {
          console.error("Stop at trigger failed:", e);
        }
      }
    }, 30_000);

    return () => clearInterval(interval);
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
    stopAt,
    setStopAt,
  };

  async function setStopAt(value: string | null) {
    try {
      await api.setStopAt(value);
      setStopAtState(value);
    } catch (e) {
      console.error("Failed to set stop_at:", e);
    }
  }

  async function runWakeup(id: string) {
    try {
      await invoke("run_wakeup", { id });
    } catch (e) {
      console.error("Failed to run:", e);
    }
  }
}
