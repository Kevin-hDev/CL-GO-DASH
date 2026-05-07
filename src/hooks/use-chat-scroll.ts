import { useState, useCallback } from "react";

interface UseChatScrollReturn {
  isAtBottom: boolean;
  scrollToBottom: () => void;
  setIsAtBottom: (value: boolean) => void;
}

export function useChatScroll(): UseChatScrollReturn {
  const [isAtBottom, setIsAtBottom] = useState(true);

  const scrollToBottom = useCallback(() => {
    setIsAtBottom(true);
  }, []);

  return { isAtBottom, scrollToBottom, setIsAtBottom };
}
