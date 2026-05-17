import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
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
let pendingFetchAll: Promise<Map<string, AvailableModel[]>> | null = null;

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
  const [catalog, configuredIds] = await Promise.all([
    invoke<ProviderSpec[]>("list_llm_providers_catalog"),
    invoke<string[]>("list_configured_providers"),
  ]);

  const configured = catalog.filter((spec) => configuredIds.includes(spec.id));

  const results = await Promise.allSettled(
    configured.map(async (spec) => {
      const models = await invoke<LlmModelInfo[]>("list_llm_models", {
        providerId: spec.id,
      });
      return { spec, models };
    }),
  );

  const result = new Map<string, AvailableModel[]>();
  for (const entry of results) {
    if (entry.status !== "fulfilled") continue;
    const { spec, models } = entry.value;
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
  }
  return result;
}

async function fetchCodexModels(): Promise<AvailableModel[]> {
  try {
    const status = await invoke<{ logged_in: boolean }>("codex_status");
    if (!status.logged_in) return [];
    const models = await invoke<LlmModelInfo[]>("codex_models");
    return models.map(
      (m): AvailableModel => ({
        id: m.id,
        provider_id: "codex-oauth",
        provider_name: "GPT (Codex)",
        is_local: false,
        supports_tools: m.supports_tools,
        supports_vision: m.supports_vision ?? false,
        supports_thinking: m.supports_thinking ?? false,
        is_free: true,
        hint: m.context_length ? `${Math.round(m.context_length / 1000)}K ctx` : undefined,
      }),
    );
  } catch {
    return [];
  }
}

async function fetchAllModels(): Promise<Map<string, AvailableModel[]>> {
  const result = new Map<string, AvailableModel[]>();

  const [ollamaResult, cloudResult, codexResult] = await Promise.allSettled([
    fetchOllamaModels(),
    fetchCloudModels(),
    fetchCodexModels(),
  ]);

  if (ollamaResult.status === "fulfilled" && ollamaResult.value.length > 0) {
    result.set("ollama", ollamaResult.value);
  }
  if (cloudResult.status === "fulfilled") {
    for (const [k, v] of cloudResult.value) result.set(k, v);
  }
  if (codexResult.status === "fulfilled" && codexResult.value.length > 0) {
    result.set("codex-oauth", codexResult.value);
  }

  cachedGroups = result;
  return result;
}

function getAllModels(): Promise<Map<string, AvailableModel[]>> {
  pendingFetchAll ??= fetchAllModels().finally(() => {
    pendingFetchAll = null;
  });
  return pendingFetchAll;
}

export function useAvailableModels() {
  const [groups, setGroups] = useState<Map<string, AvailableModel[]>>(cachedGroups);
  const [loading, setLoading] = useState(cachedGroups.size === 0);

  const refresh = useCallback(async () => {
    const result = await getAllModels();
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
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    void refresh();
    const unsubOllama = listen("ollama-models-changed", () => void refreshOllama());
    const unsubFs = listen("fs:config-changed", () => void refresh());
    const unsubStatus = listen<boolean>("ollama-status", (e) => {
      if (e.payload) setTimeout(() => void refreshOllama(), 2000);
    });
    return () => {
      cleanupTauriListener(unsubOllama);
      cleanupTauriListener(unsubFs);
      cleanupTauriListener(unsubStatus);
    };
  }, [refresh, refreshOllama]);

  return { groups, loading, refresh };
}
