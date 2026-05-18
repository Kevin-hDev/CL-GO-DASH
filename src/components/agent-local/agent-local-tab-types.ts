import type { ReactNode } from "react";
import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";

export interface TabSlots {
  list: ReactNode;
  detail: ReactNode;
}

export interface AgentLocalTabProps {
  navState: AgentLocalNavState;
  onSessionChange?: (id: string | null) => void;
  onNavChange?: (partial: DeepPartial<AgentLocalNavState>) => void;
  onNavReplace?: (partial: DeepPartial<AgentLocalNavState>) => void;
  listFocused?: boolean;
  reportContent: (slots: TabSlots) => void;
}
