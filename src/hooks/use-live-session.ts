import { useState, useCallback, useRef, useEffect } from "react";
import { useFsEventWithPayload } from "./use-fs-event";
import type { SessionEntry } from "@/types/session";

export function useLiveSession(isLive: boolean) {
  const [entries, setEntries] = useState<SessionEntry[]>([]);
  const activeRef = useRef(isLive);
  activeRef.current = isLive;

  const onNew = useCallback((payload: SessionEntry[]) => {
    if (!activeRef.current) return;
    setEntries((prev) => [...prev, ...payload]);
  }, []);

  useFsEventWithPayload<SessionEntry[]>("fs:session-message", onNew);

  useEffect(() => {
    if (!isLive) setEntries([]);
  }, [isLive]);

  return entries;
}
