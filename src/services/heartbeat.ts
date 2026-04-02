import { invoke } from "@tauri-apps/api/core";
import type { ScheduledWakeup, HeartbeatConfig } from "@/types/config";

export async function listWakeups(): Promise<ScheduledWakeup[]> {
  return invoke<ScheduledWakeup[]>("list_wakeups");
}

export async function createWakeup(params: {
  time: string;
  mode: string;
  prompt?: string;
}): Promise<ScheduledWakeup> {
  return invoke<ScheduledWakeup>("create_wakeup", params);
}

export async function updateWakeup(
  wakeup: ScheduledWakeup,
): Promise<void> {
  return invoke("update_wakeup", { wakeup });
}

export async function deleteWakeup(id: string): Promise<void> {
  return invoke("delete_wakeup", { id });
}

export async function getHeartbeatConfig(): Promise<HeartbeatConfig> {
  return invoke<HeartbeatConfig>("get_heartbeat_config");
}

export async function setHeartbeatActive(
  active: boolean,
): Promise<void> {
  return invoke("set_heartbeat_active", { active });
}

export async function setStopAt(
  stopAt: string | null,
): Promise<void> {
  return invoke("set_stop_at", { stopAt });
}
