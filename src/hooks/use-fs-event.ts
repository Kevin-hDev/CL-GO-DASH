import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";

type FsEvent =
  | "fs:config-changed"
  | "fs:personality-changed"
  | "fs:logs-changed"
  | "fs:connectors-changed"
  | "fs:skills-changed"
  | "fs:providers-changed";

export function useFsEvent(event: FsEvent, callback: () => void) {
  useEffect(() => {
    const unlisten = listen(event, callback);
    return () => { cleanupTauriListener(unlisten); };
  }, [event, callback]);
}

export function useFsEventWithPayload<T>(
  event: FsEvent,
  callback: (payload: T) => void,
) {
  useEffect(() => {
    const unlisten = listen<T>(event, (e) => callback(e.payload));
    return () => { cleanupTauriListener(unlisten); };
  }, [event, callback]);
}
