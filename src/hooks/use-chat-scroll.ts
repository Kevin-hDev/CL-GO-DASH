import { useState, useCallback, useRef } from "react";

interface UseChatScrollReturn {
  isAtBottom: boolean;
  scrollToBottom: () => void;
  setIsAtBottom: (value: boolean) => void;
  scrollActionRef: React.MutableRefObject<(() => void) | null>;
}

export function useChatScroll(): UseChatScrollReturn {
  const [isAtBottom, setIsAtBottom] = useState(true);
  const scrollActionRef = useRef<(() => void) | null>(null);

  const scrollToBottom = useCallback(() => {
    scrollActionRef.current?.();
    setIsAtBottom(true);
  }, []);

  return { isAtBottom, scrollToBottom, setIsAtBottom, scrollActionRef };
}
