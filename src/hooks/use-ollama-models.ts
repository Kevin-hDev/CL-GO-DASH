import { useState, useEffect, useCallback, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
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
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch→setState is intentional
    void refresh();
    const unlisten = listen("ollama-models-changed", () => { void refresh(); });
    return () => { cleanupTauriListener(unlisten); };
  }, [refresh]);

  const groupedByFamily = useMemo(
    () => models.reduce<Record<string, OllamaModel[]>>((acc, m) => {
      const family = m.family || "other";
      if (!acc[family]) acc[family] = [];
      acc[family].push(m);
      return acc;
    }, {}),
    [models],
  );

  return { models, groupedByFamily, loading, refresh };
}
