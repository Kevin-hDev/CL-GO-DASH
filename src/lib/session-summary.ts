import { countLines } from "./file-preview-utils";
import type {
  AgentMessage,
  AgentSession,
  AgentSessionMeta,
  AgentTodoRun,
  SubagentInfo,
  ToolActivityRecord,
} from "@/types/agent";

export interface SessionChangeSummary {
  additions: number;
  deletions: number;
  files: number;
}

export function summarizeLastRequestChanges(messages: AgentMessage[]): SessionChangeSummary {
  const message = findLastAssistantMessage(messages);
  if (!message) return { additions: 0, deletions: 0, files: 0 };

  return toolsFromMessage(message).reduce<SessionChangeSummary>((summary, tool) => {
    if (tool.is_error) return summary;
    if (tool.name === "write_file" && tool.content != null) {
      return {
        additions: summary.additions + countLines(tool.content),
        deletions: summary.deletions,
        files: summary.files + 1,
      };
    }
    if (tool.name === "edit_file" && tool.old_text != null && tool.new_text != null) {
      return {
        additions: summary.additions + countLines(tool.new_text),
        deletions: summary.deletions + countLines(tool.old_text),
        files: summary.files + 1,
      };
    }
    return summary;
  }, { additions: 0, deletions: 0, files: 0 });
}

export function activeTodoRuns(session: Pick<AgentSession, "todo_runs"> | null): AgentTodoRun[] {
  return (session?.todo_runs ?? []).filter((run) => run.status === "active");
}

export function childSubagents(parentSessionId: string, sessions: AgentSessionMeta[]): SubagentInfo[] {
  return sessions
    .filter((session) => session.parent_session_id === parentSessionId)
    .map((session) => ({
      sessionId: session.id,
      name: session.name,
      type: session.subagent_type ?? "explorer",
      status: session.subagent_status ?? "completed",
      promptPreview: "",
      runId: session.subagent_run_id,
    }));
}

function findLastAssistantMessage(messages: AgentMessage[]): AgentMessage | null {
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (messages[index].role === "assistant") return messages[index];
  }
  return null;
}

function toolsFromMessage(message: AgentMessage): ToolActivityRecord[] {
  if (message.segments && message.segments.length > 0) {
    return message.segments.flatMap((segment) => segment.tools);
  }
  return message.tool_activities ?? [];
}
