import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";

interface StreamEventPayload {
  sessionId: string;
  event: { event: string; data?: { status?: string } };
}

export function useCompression(sessionId: string) {
  const [isCompressing, setIsCompressing] = useState(false);

  useEffect(() => {
    const unlisten = listen<StreamEventPayload>("agent-stream-event", (ev) => {
      if (ev.payload.sessionId !== sessionId) return;
      if (ev.payload.event.event === "compressing") {
        setIsCompressing(ev.payload.event.data?.status === "start");
      }
    });
    return () => { cleanupTauriListener(unlisten); };
  }, [sessionId]);

  return { isCompressing };
}
