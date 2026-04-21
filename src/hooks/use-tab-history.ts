import { useState, useCallback, useRef } from "react";

export interface NavState {
  tab: string;
  settingsSubTab?: string;
  sessionId?: string | null;
  wakeupId?: string | null;
  personalityPath?: string | null;
}

const MAX_HISTORY = 50;

export function useTabHistory(initial: NavState) {
  const [current, setCurrent] = useState(initial);
  const [navIndex, setNavIndex] = useState(0);
  const history = useRef<NavState[]>([initial]);
  const live = useRef(initial);
  const navigating = useRef(false);

  const push = useCallback((partial: Partial<NavState>) => {
    if (navigating.current) return;
    const next = { ...live.current, ...partial };
    live.current = next;
    setNavIndex((i) => {
      history.current = history.current.slice(0, i + 1);
      history.current.push(next);
      if (history.current.length > MAX_HISTORY) {
        history.current.shift();
        return i;
      }
      return i + 1;
    });
    setCurrent(next);
  }, []);

  const goBack = useCallback(() => {
    setNavIndex((i) => {
      if (i <= 0) return i;
      const newIdx = i - 1;
      const state = history.current[newIdx];
      live.current = state;
      navigating.current = true;
      setCurrent(state);
      queueMicrotask(() => { navigating.current = false; });
      return newIdx;
    });
  }, []);

  const goForward = useCallback(() => {
    setNavIndex((i) => {
      if (i >= history.current.length - 1) return i;
      const newIdx = i + 1;
      const state = history.current[newIdx];
      live.current = state;
      navigating.current = true;
      setCurrent(state);
      queueMicrotask(() => { navigating.current = false; });
      return newIdx;
    });
  }, []);

  const canGoBack = navIndex > 0;
  const canGoForward = navIndex < history.current.length - 1;

  return { current, push, goBack, goForward, canGoBack, canGoForward };
}
