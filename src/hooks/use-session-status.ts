import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { SignalState } from "@/components/heartbeat/signal-dot";

interface SessionStatus {
  Idle?: null;
  Running?: { pid: number; since: string };
  Crashed?: null;
}

const POLL_INTERVAL_MS = 5000;

export function useSessionStatus(): SignalState {
  const [state, setState] = useState<SignalState>("idle");
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  useEffect(() => {
    async function check() {
      try {
        const status = await invoke<SessionStatus>("get_session_status");
        if (status.Running) {
          setState("live");
        } else if (status.Crashed) {
          setState("error");
        } else {
          setState("idle");
        }
      } catch {
        setState("idle");
      }
    }

    check();
    intervalRef.current = setInterval(check, POLL_INTERVAL_MS);

    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
      }
    };
  }, []);

  return state;
}
