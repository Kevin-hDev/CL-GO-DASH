import { invoke } from "@tauri-apps/api/core";
import type { AvailableModel } from "./use-available-models";
import type { ReasoningMode } from "@/lib/reasoning-modes";

export interface OAuthModelInfo {
  id: string;
  provider_id: "openai" | "moonshot" | "xai";
  display_name: string;
  context_length?: number;
  supports_tools: boolean;
  supports_vision: boolean;
  supports_thinking: boolean;
  reasoning_modes?: ReasoningMode[];
  interactive_only: boolean;
}

const PROVIDERS = {
  openai: { id: "codex-oauth", name: "OpenAI · OAuth" },
  moonshot: { id: "moonshot-oauth", name: "Moonshot AI · OAuth" },
  xai: { id: "xai-oauth", name: "xAI · OAuth" },
} as const;

export function mapOAuthModels(models: OAuthModelInfo[]): Map<string, AvailableModel[]> {
  const groups = new Map<string, AvailableModel[]>();
  for (const model of models) {
    const provider = PROVIDERS[model.provider_id];
    const mapped: AvailableModel = {
      id: model.id,
      provider_id: provider.id,
      provider_name: provider.name,
      auth_source: "oauth",
      is_local: false,
      supports_tools: model.supports_tools,
      supports_vision: model.supports_vision,
      supports_thinking: model.supports_thinking,
      reasoning_modes: model.reasoning_modes,
      is_free: true,
      interactive_only: model.interactive_only,
      hint: model.context_length ? `${Math.round(model.context_length / 1000)}K ctx` : undefined,
    };
    groups.set(provider.id, [...(groups.get(provider.id) ?? []), mapped]);
  }
  return groups;
}

export async function fetchOAuthModels(): Promise<Map<string, AvailableModel[]>> {
  const models = await invoke<OAuthModelInfo[]>("list_oauth_provider_models");
  return mapOAuthModels(models);
}
