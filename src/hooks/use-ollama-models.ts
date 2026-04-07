import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { OllamaModel } from "@/types/agent";

export function useOllamaModels() {
  const [models, setModels] = useState<OllamaModel[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const list = await invoke<OllamaModel[]>("list_ollama_models");
      setModels(list);
    } catch {
      setModels([]);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
    const unlisten = listen("ollama-models-changed", () => { refresh(); });
    return () => { unlisten.then((fn) => fn()).catch(() => {}); };
  }, [refresh]);

  const groupedByFamily = models.reduce<Record<string, OllamaModel[]>>((acc, m) => {
    const family = m.family || "other";
    if (!acc[family]) acc[family] = [];
    acc[family].push(m);
    return acc;
  }, {});

  return { models, groupedByFamily, loading, refresh };
}
