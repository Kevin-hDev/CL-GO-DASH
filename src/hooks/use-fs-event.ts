import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

type FsEvent =
  | "fs:config-changed"
  | "fs:personality-changed"
  | "fs:logs-changed";

export function useFsEvent(event: FsEvent, callback: () => void) {
  useEffect(() => {
    const unlisten = listen(event, callback);
    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [event, callback]);
}

export function useFsEventWithPayload<T>(
  event: FsEvent,
  callback: (payload: T) => void,
) {
  useEffect(() => {
    const unlisten = listen<T>(event, (e) => callback(e.payload));
    return () => {
      unlisten.then((fn) => fn()).catch(() => {});
    };
  }, [event, callback]);
}
