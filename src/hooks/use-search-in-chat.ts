import { useState, useCallback } from "react";
import type { AgentMessage } from "@/types/agent";

export function useSearchInChat(messages: AgentMessage[]) {
  const [query, setQuery] = useState("");
  const [visible, setVisible] = useState(false);

  const matches = query.trim()
    ? messages.filter((m) =>
        m.content.toLowerCase().includes(query.toLowerCase()),
      )
    : [];

  const toggle = useCallback(() => {
    setVisible((v) => !v);
    if (visible) setQuery("");
  }, [visible]);

  return { query, setQuery, visible, toggle, matches, matchCount: matches.length };
}
