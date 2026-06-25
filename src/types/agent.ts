export interface OllamaModel {
  name: string;
  size: number;
  family: string;
  parameter_size: string;
  quantization: string;
  architecture: string;
  is_moe: boolean;
  context_length: number;
  capabilities: ("completion" | "vision" | "thinking" | "tools")[];
  digest_short: string;
  aliases: string[];
  is_customized: boolean;
}

export interface RegistryModelDetails {
  name: string;
  description_short: string;
  description_long_markdown: string;
  capabilities: string[];
  sizes: string[];
  context_length: number | null;
}

export interface RegistryTag {
  name: string;
  digest_short: string;
  size_gb: number | null;
  context_length: number | null;
}

export interface ModelInfo {
  name: string;
  modelfile: string;
  parameters: string;
  template: string;
  family: string;
  parameter_size: string;
  quantization: string;
  architecture: string;
  is_moe: boolean;
  context_length: number;
  capabilities: string[];
  has_audio: boolean;
  license: string;
}

export interface ModelProfile {
  name: string;
  parameter_size: string;
  file_size: number;
  architecture: string;
  context_length: number;
  family: string;
  quantization: string;
  capabilities: string[];
  is_moe: boolean;
  has_audio: boolean;
  license: string;
}

export interface RegistryModel {
  name: string;
  description: string;
  tags: string[];
  is_installed: boolean;
}

export interface Project {
  id: string;
  name: string;
  path: string;
  order: number;
  created_at: string;
}

export interface AgentSession {
  id: string;
  name: string;
  created_at: string;
  model: string;
  /** Provider (ex: "ollama", "groq", "google", …). Défaut "ollama". */
  provider: string;
  thinking_enabled: boolean;
  reasoning_mode?: string;
  accumulated_tokens: number;
  messages: AgentMessage[];
  todos?: AgentTodoItem[];
  todo_runs?: AgentTodoRun[];
  active_todo_run_id?: string;
  stream_failures?: AgentStreamFailure[];
  project_id?: string;
  working_dir?: string;
  parent_session_id?: string;
  subagent_type?: "explorer" | "coder";
  subagent_worktree?: string;
  subagent_prompt?: string;
  subagent_status?: "running" | "completed" | "failed" | "cancelled";
  subagent_run_id?: string;
}

export type AgentTodoStatus = "pending" | "in_progress" | "completed";
export type AgentTodoRunStatus = "active" | "paused" | "completed" | "abandoned";

export interface AgentTodoItem {
  content: string;
  active_form?: string;
  status: AgentTodoStatus;
}

export interface AgentTodoRun {
  id: string;
  title: string;
  status: AgentTodoRunStatus;
  todos: AgentTodoItem[];
  created_at: string;
  updated_at: string;
  paused_reason?: string;
}

export interface AgentStreamFailure {
  code: string;
  occurred_at: string;
  is_connection: boolean;
  active_todo_run_id?: string;
  active_todo_title?: string;
}

export interface AgentErrorDiagnosticSummary {
  requestId: string;
  phase: string;
  errorType: string;
  lastToolName?: string;
  safeSummary: string;
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

export interface AgentMessage {
  id: string;
  role: "user" | "assistant" | "tool";
  content: string;
  thinking?: string;
  tool_calls?: ToolCallRequest[];
  tool_name?: string;
  tool_activities?: ToolActivityRecord[];
  segments?: SavedSegment[];
  files: FileAttachment[];
  timestamp: string;
  skill_names?: string[];
  tokens?: number;
}

export interface SavedSegment {
  thinking?: string;
  tools: ToolActivityRecord[];
  content: string;
}

export interface ToolActivityRecord {
  name: string;
  summary: string;
  args?: Record<string, unknown>;
  result?: string;
  is_error?: boolean;
  content?: string;
  old_text?: string;
  new_text?: string;
  start_line?: number;
}

export interface ToolCallRequest {
  extra_content?: unknown;
  function: { name: string; arguments: Record<string, unknown> };
}

export interface FileAttachment {
  name: string;
  path: string;
  mime_type: string;
  size: number;
  thumbnail?: string;
}

export interface TabState {
  tabs: TabInfo[];
  active_index: number;
}

export interface TabInfo {
  session_id: string;
  label: string;
}

export interface ToolResult {
  content: string;
  is_error: boolean;
  truncated?: boolean;
}

export interface SearchResult {
  title: string;
  url: string;
  snippet: string;
}

export interface SkillInfo {
  name: string;
  description: string;
  path: string;
  source: string;
}

export interface PullProgress {
  status: string;
  completed?: number;
  total?: number;
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
  | { event: "todoUpdated"; data: { todos: AgentTodoItem[] } };
