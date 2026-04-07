import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ModelInfo } from "@/types/agent";

interface ContextBarState {
  usedTokens: number;
  maxTokens: number;
  percentage: number;
  color: string;
}

export function useContextBar(model: string, accumulatedTokens: number) {
  const [maxTokens, setMaxTokens] = useState(4096);

  const fetchContextLength = useCallback(async () => {
    try {
      const info = await invoke<ModelInfo>("show_ollama_model", { name: model });
      setMaxTokens(info.context_length || 4096);
    } catch (e) {
      console.warn("Impossible de charger le contexte du modèle:", e);
    }
  }, [model]);

  useEffect(() => {
    if (model) fetchContextLength();
  }, [model, fetchContextLength]);

  useEffect(() => {
    const unlisten = listen("modelfile-updated", () => { fetchContextLength(); });
    return () => { unlisten.then((fn) => fn()).catch(() => {}); };
  }, [fetchContextLength]);

  const percentage = maxTokens > 0 ? (accumulatedTokens / maxTokens) * 100 : 0;

  const color = percentage > 95
    ? "#dc2626"
    : percentage > 70
    ? "#ea580c"
    : percentage > 60
    ? "#ca8a04"
    : "#166534";

  const state: ContextBarState = {
    usedTokens: accumulatedTokens,
    maxTokens,
    percentage,
    color,
  };

  return state;
}
