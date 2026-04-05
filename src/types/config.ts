export interface ClgoConfig {
  version: string;
  projects_root: string;
  claude_projects: string;
  heartbeat: HeartbeatConfig;
  communication: CommunicationConfig;
  hooks: HooksConfig;
  scheduled_wakeups: ScheduledWakeup[];
  scheduled_tasks: unknown[];
  rube_usage: RubeUsage | null;
}

export interface HeartbeatConfig {
  active: boolean;
  mode: string;
  stop_at: string | null;
  interval_minutes: number;
  started_at: string | null;
}

export interface CommunicationConfig {
  provider: string;
  chat_id: string;
}

export interface HooksConfig {
  post_explorer: HookEntry[];
  post_auto: HookEntry[];
}

export interface HookEntry {
  name: string;
  command: string[];
  cwd: string | null;
}

export interface ScheduledWakeup {
  id: string;
  time: string;
  mode: string;
  prompt: string | null;
  name: string | null;
  active: boolean;
}

export interface RubeUsage {
  month: string | null;
  count: number;
  limit: number;
}
