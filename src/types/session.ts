export interface SessionMeta {
  id: string;
  file_path: string;
  start: string;
  end: string;
  duration_minutes: number;
  mode: string;
  message_count: number;
  version: string;
  custom_name: string | null;
}

export interface SessionDetail {
  meta: SessionMeta;
  entries: SessionEntry[];
  messages: SessionMessage[];
  files_modified: string[];
  tools_used: string[];
}

export type SessionEntry =
  | { kind: "message" } & SessionMessage
  | { kind: "tool" } & ToolCall;

export interface SessionMessage {
  role: "user" | "assistant";
  content: string;
  timestamp: string;
}

export interface ToolCall {
  name: string;
  summary: string;
  timestamp: string;
  old_text?: string;
  new_text?: string;
}
