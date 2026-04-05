import { useState, useCallback, useRef, useEffect } from "react";
import { useFsEventWithPayload } from "./use-fs-event";

interface LiveMessage {
  role: string;
  content: string;
  timestamp: string;
}

export function useLiveSession(isLive: boolean) {
  const [messages, setMessages] = useState<LiveMessage[]>([]);
  const activeRef = useRef(isLive);
  activeRef.current = isLive;

  const onNewMessages = useCallback((payload: LiveMessage[]) => {
    if (!activeRef.current) return;
    setMessages((prev) => [...prev, ...payload]);
  }, []);

  useFsEventWithPayload<LiveMessage[]>("fs:session-message", onNewMessages);

  // Reset when session state changes
  useEffect(() => {
    if (!isLive) setMessages([]);
  }, [isLive]);

  return messages;
}
