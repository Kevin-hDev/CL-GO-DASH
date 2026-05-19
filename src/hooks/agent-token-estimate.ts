import type { AgentMessage, AgentSession } from "@/types/agent";

const CHARS_PER_TOKEN = 4;

export function estimateAgentMessagesTokens(messages: AgentMessage[]): number {
  return messages.reduce((sum, message) => sum + estimateMessage(message), 0);
}

export function resolveSessionTokenCount(session: AgentSession): number {
  return Math.max(session.accumulated_tokens, estimateAgentMessagesTokens(session.messages));
}

function estimateMessage(message: AgentMessage): number {
  let chars = message.content.length;
  chars += message.thinking?.length ?? 0;
  if (message.tool_calls) {
    for (const call of message.tool_calls) {
      chars += call.function.name.length;
      chars += JSON.stringify(call.function.arguments).length;
    }
  }
  if (message.tool_activities) {
    for (const activity of message.tool_activities) {
      chars += activity.summary.length;
      chars += activity.result?.length ?? 0;
      chars += activity.content?.length ?? 0;
    }
  }
  return Math.floor(chars / CHARS_PER_TOKEN);
}
