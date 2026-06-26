import type { AgentMessage } from "./agent-message";
import type { AgentPlanApprovalDecision, AgentPlanRun, AgentPlanWorkflowStatus } from "./agent-plan";
import type { AgentTodoItem, AgentTodoRun } from "./agent-todo";

export interface Project {
  id: string;
  name: string;
  path: string;
  order: number;
  created_at: string;
}

export interface AgentStreamFailure {
  code: string;
  occurred_at: string;
  is_connection: boolean;
  active_todo_run_id?: string;
  active_todo_title?: string;
}

export interface AgentSession {
  id: string;
  name: string;
  created_at: string;
  model: string;
  provider: string;
  thinking_enabled: boolean;
  reasoning_mode?: string;
  accumulated_tokens: number;
  messages: AgentMessage[];
  todos?: AgentTodoItem[];
  todo_runs?: AgentTodoRun[];
  active_todo_run_id?: string;
  stream_failures?: AgentStreamFailure[];
  plan_mode_enabled?: boolean;
  plan_runs?: AgentPlanRun[];
  active_plan_id?: string;
  plan_workflow_status?: AgentPlanWorkflowStatus;
  plan_approval_decision?: AgentPlanApprovalDecision;
  project_id?: string;
  working_dir?: string;
  parent_session_id?: string;
  subagent_type?: "explorer" | "coder";
  subagent_worktree?: string;
  subagent_prompt?: string;
  subagent_status?: "running" | "completed" | "failed" | "cancelled";
  subagent_run_id?: string;
}

export interface AgentSessionMeta {
  id: string;
  name: string;
  created_at: string;
  model: string;
  provider: string;
  thinking_enabled?: boolean;
  reasoning_mode?: string;
  message_count: number;
  project_id?: string;
  parent_session_id?: string;
  subagent_type?: "explorer" | "coder";
  subagent_status?: "running" | "completed" | "failed" | "cancelled";
  subagent_run_id?: string;
  is_gateway?: boolean;
  gateway_channel_key?: string;
}

export interface SubagentInfo {
  sessionId: string;
  name: string;
  type: "explorer" | "coder";
  status: "running" | "completed" | "failed" | "cancelled";
  promptPreview: string;
  runId?: string;
  spawnedAt?: number;
}

export interface TabState {
  tabs: TabInfo[];
  active_index: number;
}

export interface TabInfo {
  session_id: string;
  label: string;
}
