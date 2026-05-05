import type { ReactNode } from "react";

export interface TabSlots {
  list: ReactNode;
  detail: ReactNode;
}

export interface AgentLocalTabProps {
  requestedSessionId?: string | null;
  onSessionChange?: (id: string | null) => void;
  listFocused?: boolean;
  reportContent: (slots: TabSlots) => void;
}
