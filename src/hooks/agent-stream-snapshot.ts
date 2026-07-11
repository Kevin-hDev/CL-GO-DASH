import { estimateAgentMessagesTokens } from "./agent-token-estimate";
import { markSubagentSnapshot } from "./agent-stream-persistence-owner";
import type { StreamRecord } from "./agent-stream-cleanup";
import type { AgentMessage } from "@/types/agent";

export function applySessionSnapshot(record: StreamRecord, messages: AgentMessage[]) {
  markSubagentSnapshot(record);
  record.state = {
    ...record.state,
    messages,
    sessionTokenCount: estimateAgentMessagesTokens(messages),
    sessionTokenCountEstimated: true,
    isStreaming: true,
    activeStreamItem: null,
    persisted: false,
    completed: false,
  };
}
