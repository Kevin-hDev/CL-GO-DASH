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
  messages: SessionMessage[];
  files_modified: string[];
  tools_used: string[];
}

export interface SessionMessage {
  role: "user" | "assistant";
  content: string;
  timestamp: string;
}
