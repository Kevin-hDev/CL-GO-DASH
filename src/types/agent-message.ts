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
  work_duration_ms?: number;
  /** Marqueur frontend temporaire : ce bloc appartient encore au stream actif. */
  is_stream_checkpoint?: boolean;
}

export interface SavedSegment {
  thinking?: string;
  tools: ToolActivityRecord[];
  content: string;
  phase?: "work" | "final";
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
  resolved_path?: string;
  affected_paths?: string[];
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

export interface ToolResult {
  content: string;
  is_error: boolean;
  truncated?: boolean;
  affected_paths?: string[];
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
