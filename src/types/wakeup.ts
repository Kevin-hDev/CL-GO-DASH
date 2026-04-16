export type WakeupSchedule =
  | { kind: "once"; datetime: string }
  | { kind: "daily"; time: string }
  | { kind: "weekly"; weekday: number; time: string };

export interface ScheduledWakeup {
  id: string;
  name: string;
  model: string;
  provider: string;
  prompt: string;
  schedule: WakeupSchedule;
  description: string;
  active: boolean;
  paused_by_global: boolean;
  created_at: string;
}

export interface CreateWakeupInput {
  name: string;
  model: string;
  provider: string;
  prompt: string;
  schedule: WakeupSchedule;
  description: string;
}

export interface HeartbeatConfig {
  global_paused: boolean;
}
