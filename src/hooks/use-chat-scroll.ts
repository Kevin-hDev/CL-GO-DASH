import { useState, useRef, useCallback, useEffect } from "react";

interface UseChatScrollParams {
  messagesLength: number;
  currentContent: string;
  currentThinking: string;
  currentToolsLength: number;
}

interface UseChatScrollReturn {
  scrollRef: React.RefObject<HTMLDivElement | null>;
  bottomRef: React.RefObject<HTMLDivElement | null>;
  isAtBottom: boolean;
  scrollToBottom: () => void;
  handleScroll: () => void;
}

export function useChatScroll({
  messagesLength,
  currentContent,
  currentThinking,
  currentToolsLength,
}: UseChatScrollParams): UseChatScrollReturn {
  const scrollRef = useRef<HTMLDivElement>(null);
  const bottomRef = useRef<HTMLDivElement>(null);
  const [isAtBottom, setIsAtBottom] = useState(true);
  const following = useRef(true);

  const handleScroll = useCallback(() => {
    const el = scrollRef.current;
    if (!el) return;
    const atBottom = el.scrollHeight - el.scrollTop - el.clientHeight < 80;
    setIsAtBottom(atBottom);
    if (atBottom) following.current = true;
  }, []);

  useEffect(() => {
    const el = scrollRef.current;
    if (!el) return;
    const onWheel = (e: WheelEvent) => {
      if (e.deltaY < 0) following.current = false;
    };
    el.addEventListener("wheel", onWheel, { passive: true });
    return () => el.removeEventListener("wheel", onWheel);
  }, []);

  const scrollToBottom = useCallback(() => {
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
    following.current = true;
  }, []);

  useEffect(() => {
    following.current = true;
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [messagesLength]);

  useEffect(() => {
    if (!following.current) return;
    const el = scrollRef.current;
    if (el) el.scrollTop = el.scrollHeight;
  }, [currentContent, currentThinking, currentToolsLength]);

  return { scrollRef, bottomRef, isAtBottom, scrollToBottom, handleScroll };
}
