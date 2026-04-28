export interface AgentLocalTabProps {
  requestedSessionId?: string | null;
  onSessionChange?: (id: string | null) => void;
}
