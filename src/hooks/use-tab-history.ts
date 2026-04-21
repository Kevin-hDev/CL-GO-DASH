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
  const history = useRef<NavState[]>([initial]);
  const index = useRef(0);
  const live = useRef(initial);

  const push = useCallback((partial: Partial<NavState>) => {
    const next = { ...live.current, ...partial };
    live.current = next;
    const i = index.current;
    history.current = history.current.slice(0, i + 1);
    history.current.push(next);
    if (history.current.length > MAX_HISTORY) {
      history.current.shift();
    } else {
      index.current = i + 1;
    }
    setCurrent(next);
  }, []);

  const goBack = useCallback(() => {
    if (index.current <= 0) return;
    index.current -= 1;
    const state = history.current[index.current];
    live.current = state;
    setCurrent(state);
  }, []);

  const goForward = useCallback(() => {
    if (index.current >= history.current.length - 1) return;
    index.current += 1;
    const state = history.current[index.current];
    live.current = state;
    setCurrent(state);
  }, []);

  const canGoBack = index.current > 0;
  const canGoForward = index.current < history.current.length - 1;

  return { current, push, goBack, goForward, canGoBack, canGoForward };
}
