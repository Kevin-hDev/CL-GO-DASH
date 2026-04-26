import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";

export type PermissionMode = "auto" | "manual" | "chat";

interface AgentSettings {
  permission_mode: PermissionMode;
}

const sessionModes = new Map<string, PermissionMode>();
let defaultMode: PermissionMode = "auto";

export function usePermissionMode(sessionId?: string) {
  const resolvedMode = sessionId && sessionModes.has(sessionId)
    ? sessionModes.get(sessionId)!
    : defaultMode;

  const [mode, setMode] = useState<PermissionMode>(resolvedMode);
  const [loaded, setLoaded] = useState(false);
  const sessionRef = useRef(sessionId);
  sessionRef.current = sessionId;

  useEffect(() => {
    invoke<AgentSettings>("get_agent_settings")
      .then((s) => {
        defaultMode = s.permission_mode;
        if (!sessionId || !sessionModes.has(sessionId)) {
          setMode(defaultMode);
        }
        setLoaded(true);
      })
      .catch(() => setLoaded(true));
  }, []);

  useEffect(() => {
    if (!sessionId) return;
    const stored = sessionModes.get(sessionId);
    if (stored) {
      setMode(stored);
    } else {
      setMode(defaultMode);
    }
  }, [sessionId]);

  const change = useCallback(async (next: PermissionMode) => {
    setMode(next);
    const sid = sessionRef.current;
    if (sid) {
      sessionModes.set(sid, next);
    }
    defaultMode = next;
    try {
      await invoke("set_permission_mode", { mode: next });
    } catch (e) {
      console.error("set_permission_mode:", e);
    }
  }, []);

  const toggle = useCallback(() => {
    const cycle: PermissionMode[] = ["chat", "manual", "auto"];
    const idx = cycle.indexOf(mode);
    change(cycle[(idx + 1) % cycle.length]);
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
