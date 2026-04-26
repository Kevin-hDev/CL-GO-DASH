import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { ModelInfo } from "@/types/agent";

export interface ContextProgressState {
  used: number;
  max: number;
}

function parseNumCtxFromModelfile(modelfile: string): number | null {
  const m = modelfile.match(/^PARAMETER\s+num_ctx\s+(\d+)/mi);
  return m ? parseInt(m[1], 10) : null;
}

interface LlmModelInfo {
  id: string;
  context_length?: number;
}

export function useContextProgress(
  model: string,
  usedTokens: number,
  provider: string = "ollama",
): ContextProgressState {
  const [max, setMax] = useState(0);

  const refresh = useCallback(async () => {
    if (!model) { setMax(0); return; }

    if (provider === "ollama") {
      try {
        const info = await invoke<ModelInfo>("show_ollama_model", { name: model });
        const fromModelfile = parseNumCtxFromModelfile(info.modelfile);
        if (fromModelfile) {
          setMax(fromModelfile);
          return;
        }
        const modelCtx = info.context_length ?? 0;
        const effectiveCtx = await invoke<number>("get_effective_context_length").catch(() => 0);
        if (effectiveCtx > 0 && modelCtx > 0) {
          setMax(Math.min(modelCtx, effectiveCtx));
        } else {
          setMax(effectiveCtx || modelCtx);
        }
      } catch {
        setMax(0);
      }
    } else {
      try {
        const models = await invoke<LlmModelInfo[]>("list_llm_models", { providerId: provider });
        const found = models.find((m) => m.id === model);
        setMax(found?.context_length ?? 0);
      } catch {
        setMax(0);
      }
    }
  }, [model, provider]);

  useEffect(() => { refresh(); }, [refresh]);

  useEffect(() => {
    const unlisten = listen("modelfile-updated", () => { refresh(); });
    return () => { unlisten.then((fn) => fn()).catch(() => {}); };
  }, [refresh]);

  return { used: usedTokens, max };
}
