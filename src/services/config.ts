import { invoke } from "@tauri-apps/api/core";
import type { ClgoConfig } from "@/types/config";

export async function getConfig(): Promise<ClgoConfig> {
  return invoke<ClgoConfig>("get_config");
}

export async function saveConfig(config: ClgoConfig): Promise<void> {
  return invoke("save_config", { config });
}
