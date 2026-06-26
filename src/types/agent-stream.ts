import type { AgentInteractiveChoiceRequest } from "./agent-interactive";
import type { AgentMessage } from "./agent-message";
import type { AgentPlanPreview } from "./agent-plan";
import type { AgentTodoItem } from "./agent-todo";

export interface AgentErrorDiagnosticSummary {
  requestId: string;
  phase: string;
  errorType: string;
  lastToolName?: string;
  safeSummary: string;
}

export type StreamEvent =
  | { event: "token"; data: { content: string; tokenCount: number; tps: number } }
  | { event: "thinking"; data: { content: string } }
  | { event: "toolCall"; data: { name: string; arguments: Record<string, unknown> } }
  | { event: "toolResult"; data: { name: string; content: string; isError: boolean; truncated?: boolean; toolCallIndex: number } }
  | { event: "turnEnd"; data: Record<string, never> }
  | { event: "permissionRequest"; data: { id: string; toolName: string; arguments: Record<string, unknown> } }
  | { event: "done"; data: { evalCount: number; evalDurationNs: number; finalTps: number; promptTokens: number; contextTokens: number } }
  | { event: "error"; data: { message: string; isConnection?: boolean; diagnostic?: AgentErrorDiagnosticSummary } }
  | { event: "notice"; data: { messageKey: string } }
  | { event: "compressing"; data: { status: string } }
  | { event: "compressionComplete"; data: Record<string, never> }
  | { event: "sessionSnapshot"; data: { messages: AgentMessage[]; tokenCount: number } }
  | { event: "subagentSpawned"; data: { subagentSessionId: string; subagentName: string; subagentType: string; promptPreview: string; runId?: string } }
  | { event: "subagentCompleted"; data: { subagentSessionId: string; success: boolean; status: "completed" | "failed" | "cancelled"; summary: string; allDone: boolean; runId?: string } }
  | { event: "todoUpdated"; data: { todos: AgentTodoItem[] } }
  | { event: "planPreviewUpdated"; data: { plan: AgentPlanPreview | null } }
  | { event: "planModeUpdated"; data: { enabled: boolean } }
  | { event: "interactiveChoiceRequest"; data: AgentInteractiveChoiceRequest };
