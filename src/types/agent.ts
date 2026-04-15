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

export interface AgentSession {
  id: string;
  name: string;
  created_at: string;
  model: string;
  thinking_enabled: boolean;
  accumulated_tokens: number;
  messages: AgentMessage[];
}

export interface AgentSessionMeta {
  id: string;
  name: string;
  created_at: string;
  model: string;
  message_count: number;
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
  result?: string;
  is_error?: boolean;
  content?: string;
  old_text?: string;
  new_text?: string;
}

export interface ToolCallRequest {
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
  | { event: "toolResult"; data: { name: string; content: string; isError: boolean } }
  | { event: "turnEnd"; data: Record<string, never> }
  | { event: "done"; data: { evalCount: number; evalDurationNs: number; finalTps: number; promptTokens: number } }
  | { event: "error"; data: { message: string } };
