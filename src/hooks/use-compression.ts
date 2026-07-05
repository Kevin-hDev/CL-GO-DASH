import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";

interface StreamEventPayload {
  sessionId: string;
  event: { event: string; data?: { status?: string } };
}

export function useCompression(sessionId: string) {
  const [state, setState] = useState({ sessionId: "", isCompressing: false });

  useEffect(() => {
    const unlisten = listen<StreamEventPayload>("agent-stream-event", (ev) => {
      if (ev.payload.sessionId !== sessionId) return;
      if (ev.payload.event.event === "compressing") {
        setState({
          sessionId,
          isCompressing: ev.payload.event.data?.status === "start",
        });
      }
      if (["compressionComplete", "done", "error", "turnEnd"].includes(ev.payload.event.event)) {
        setState({ sessionId, isCompressing: false });
      }
    });
    return () => { cleanupTauriListener(unlisten); };
  }, [sessionId]);

  return { isCompressing: state.sessionId === sessionId && state.isCompressing };
}
