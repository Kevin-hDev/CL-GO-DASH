import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

type FsEvent =
  | "fs:config-changed"
  | "fs:sessions-changed"
  | "fs:personality-changed"
  | "fs:logs-changed";

export function useFsEvent(event: FsEvent, callback: () => void) {
  useEffect(() => {
    const unlisten = listen(event, callback);
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [event, callback]);
}
