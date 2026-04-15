import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export type PermissionMode = "auto" | "manual";

interface AgentSettings {
  permission_mode: PermissionMode;
}

export function usePermissionMode() {
  const [mode, setMode] = useState<PermissionMode>("auto");
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    invoke<AgentSettings>("get_agent_settings")
      .then((s) => {
        setMode(s.permission_mode);
        setLoaded(true);
      })
      .catch((e) => {
        console.warn("get_agent_settings:", e);
        setLoaded(true);
      });
  }, []);

  const change = useCallback(async (next: PermissionMode) => {
    setMode(next);
    try {
      await invoke("set_permission_mode", { mode: next });
    } catch (e) {
      console.error("set_permission_mode:", e);
    }
  }, []);

  const toggle = useCallback(() => {
    change(mode === "auto" ? "manual" : "auto");
  }, [mode, change]);

  useEffect(() => {
    if (!loaded) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.shiftKey && e.key.startsWith("Tab")) {
        const target = e.target as HTMLElement | null;
        const tag = target?.tagName;
        if (tag === "INPUT" || tag === "TEXTAREA" || target?.isContentEditable) {
          return;
        }
        e.preventDefault();
        toggle();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [loaded, toggle]);

  return { mode, change, toggle, loaded };
}
