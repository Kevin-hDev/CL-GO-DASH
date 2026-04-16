import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  CreateWakeupInput,
  HeartbeatConfig,
  ScheduledWakeup,
} from "@/types/wakeup";
import { useFsEvent } from "./use-fs-event";

export function useWakeups() {
  const [wakeups, setWakeups] = useState<ScheduledWakeup[]>([]);
  const [globalPaused, setGlobalPaused] = useState(false);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const [list, hb] = await Promise.all([
        invoke<ScheduledWakeup[]>("list_wakeups"),
        invoke<HeartbeatConfig>("get_heartbeat_config"),
      ]);
      setWakeups(list);
      setGlobalPaused(hb.global_paused);
    } catch {
      setWakeups([]);
      setGlobalPaused(false);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  useFsEvent("fs:config-changed", refresh);

  useEffect(() => {
    const unlisten = listen("wakeup-completed", () => {
      refresh();
    });
    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [refresh]);

  const create = useCallback(
    async (input: CreateWakeupInput) => {
      const w = await invoke<ScheduledWakeup>("create_wakeup", { input });
      await refresh();
      return w;
    },
    [refresh],
  );

  const update = useCallback(
    async (wakeup: ScheduledWakeup) => {
      await invoke("update_wakeup", { wakeup });
      await refresh();
    },
    [refresh],
  );

  const remove = useCallback(
    async (id: string) => {
      await invoke("delete_wakeup", { id });
      await refresh();
    },
    [refresh],
  );

  const toggle = useCallback(
    async (id: string, active: boolean) => {
      await invoke("set_wakeup_active", { id, active });
      await refresh();
    },
    [refresh],
  );

  const setPaused = useCallback(
    async (paused: boolean) => {
      await invoke("set_global_paused", { paused });
      await refresh();
    },
    [refresh],
  );

  return {
    wakeups,
    globalPaused,
    loading,
    refresh,
    create,
    update,
    remove,
    toggle,
    setPaused,
  };
}
