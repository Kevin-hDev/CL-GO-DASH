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
  /** Libellé secondaire (taille modèle, context, etc.) */
  hint?: string;
}

interface LlmModelInfo {
  id: string;
  owned_by?: string;
  context_length?: number;
  supports_tools: boolean;
}

/**
 * Agrège tous les modèles disponibles :
 * - Ollama (modèles installés localement)
 * - Providers LLM configurés (Groq, Gemini, Mistral, …)
 *
 * Les modèles sont regroupés par provider dans l'ordre Ollama → alphabétique.
 */
export function useAvailableModels() {
  const [groups, setGroups] = useState<Map<string, AvailableModel[]>>(new Map());
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    const result = new Map<string, AvailableModel[]>();

    // 1. Ollama
    try {
      const ollamaModels = await invoke<OllamaModel[]>("list_ollama_models");
      const mapped = ollamaModels.map(
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
      if (mapped.length > 0) result.set("ollama", mapped);
    } catch {
      // Ollama non démarré — on ignore
    }

    // 2. Providers LLM configurés
    try {
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
              hint: m.context_length ? `${Math.round(m.context_length / 1000)}K ctx` : undefined,
            }),
          );
          if (mapped.length > 0) result.set(spec.id, mapped);
        } catch (e) {
          console.warn(`[models] ${spec.id}:`, e);
        }
      }
    } catch (e) {
      console.warn("[models] catalog:", e);
    }

    setGroups(result);
    setLoading(false);
  }, []);

  useEffect(() => {
    refresh();
    const unsubOllama = listen("ollama-models-changed", refresh);
    const unsubFs = listen("fs:config-changed", refresh);
    return () => {
      unsubOllama.then((f) => f()).catch(() => {});
      unsubFs.then((f) => f()).catch(() => {});
    };
  }, [refresh]);

  return { groups, loading, refresh };
}
