import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { OllamaModel } from "@/types/agent";
import type { ProviderSpec } from "@/types/api";

export interface AvailableModel {
  id: string;
  provider_id: string;
  provider_name: string;
  is_local: boolean;
  supports_tools: boolean;
  supports_vision?: boolean;
  supports_thinking?: boolean;
  is_free?: boolean;
  hint?: string;
}

interface LlmModelInfo {
  id: string;
  owned_by?: string;
  context_length?: number;
  supports_tools: boolean;
  supports_vision?: boolean;
  supports_thinking?: boolean;
  is_free?: boolean;
}

let cachedGroups: Map<string, AvailableModel[]> = new Map();

async function fetchOllamaModels(): Promise<AvailableModel[]> {
  const ollamaModels = await invoke<OllamaModel[]>("list_ollama_models");
  return ollamaModels.map(
    (m): AvailableModel => ({
      id: m.name,
      provider_id: "ollama",
      provider_name: "Ollama",
      is_local: true,
      supports_tools: m.capabilities?.includes("tools") ?? false,
      supports_vision: m.capabilities?.includes("vision") ?? false,
      supports_thinking: m.capabilities?.includes("thinking") ?? false,
      hint: m.parameter_size,
    }),
  );
}

async function fetchCloudModels(): Promise<Map<string, AvailableModel[]>> {
  const result = new Map<string, AvailableModel[]>();
  const [catalog, configuredIds] = await Promise.all([
    invoke<ProviderSpec[]>("list_llm_providers_catalog"),
    invoke<string[]>("list_configured_providers"),
  ]);

  for (const spec of catalog) {
    if (!configuredIds.includes(spec.id)) continue;
    try {
      const models = await invoke<LlmModelInfo[]>("list_llm_models", {
        providerId: spec.id,
      });
      const mapped = models.map(
        (m): AvailableModel => ({
          id: m.id,
          provider_id: spec.id,
          provider_name: spec.display_name,
          is_local: false,
          supports_tools: m.supports_tools,
          supports_vision: m.supports_vision ?? false,
          supports_thinking: m.supports_thinking ?? false,
          is_free: m.is_free ?? false,
          hint: m.context_length ? `${Math.round(m.context_length / 1000)}K ctx` : undefined,
        }),
      );
      if (mapped.length > 0) result.set(spec.id, mapped);
    } catch (e) {
      console.warn(`[models] ${spec.id}:`, e);
    }
  }
  return result;
}

async function fetchAllModels(): Promise<Map<string, AvailableModel[]>> {
  const result = new Map<string, AvailableModel[]>();

  try {
    const ollama = await fetchOllamaModels();
    if (ollama.length > 0) result.set("ollama", ollama);
  } catch {
    // Ollama non démarré
  }

  try {
    const cloud = await fetchCloudModels();
    for (const [k, v] of cloud) result.set(k, v);
  } catch (e) {
    console.warn("[models] catalog:", e);
  }

  cachedGroups = result;
  return result;
}

export function useAvailableModels() {
  const [groups, setGroups] = useState<Map<string, AvailableModel[]>>(cachedGroups);
  const [loading, setLoading] = useState(cachedGroups.size === 0);

  const refresh = useCallback(async () => {
    const result = await fetchAllModels();
    setGroups(result);
    setLoading(false);
  }, []);

  const refreshOllama = useCallback(async () => {
    try {
      const ollama = await fetchOllamaModels();
      setGroups((prev) => {
        const next = new Map(prev);
        if (ollama.length > 0) next.set("ollama", ollama);
        else next.delete("ollama");
        cachedGroups = next;
        return next;
      });
    } catch {
      setGroups((prev) => {
        const next = new Map(prev);
        next.delete("ollama");
        cachedGroups = next;
        return next;
      });
    }
  }, []);

  useEffect(() => {
    refresh();
    const unsubOllama = listen("ollama-models-changed", refreshOllama);
    const unsubFs = listen("fs:config-changed", refresh);
    return () => {
      unsubOllama.then((f) => f()).catch(() => {});
      unsubFs.then((f) => f()).catch(() => {});
    };
  }, [refresh, refreshOllama]);

  return { groups, loading, refresh };
}
