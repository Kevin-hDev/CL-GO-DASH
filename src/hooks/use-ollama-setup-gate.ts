import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import {
  hasSkippedOllamaSetup,
  ollamaSetupSkippedPatch,
  shouldShowOllamaSetup,
} from "@/lib/ollama-setup-gate";

export function useOllamaSetupGate() {
  const [showOllamaSetup, setShowOllamaSetup] = useState<boolean | null>(null);

  useEffect(() => {
    const loadOllamaGate = async () => {
      try {
        const [installed, settings] = await Promise.all([
          invoke<boolean>("is_ollama_installed"),
          invoke<Record<string, unknown>>("get_advanced_settings"),
        ]);
        setShowOllamaSetup(shouldShowOllamaSetup({
          installed,
          skipped: hasSkippedOllamaSetup(settings),
        }));
      } catch {
        setShowOllamaSetup(false);
      }
    };
    void loadOllamaGate();
  }, []);

  const completeOllamaSetup = useCallback(() => {
    invoke("patch_advanced_settings", { patch: ollamaSetupSkippedPatch(false) }).catch(() => {});
    invoke("start_ollama_sidecar").catch(() => {});
    setShowOllamaSetup(false);
  }, []);

  const skipOllamaSetup = useCallback(async () => {
    await invoke("patch_advanced_settings", { patch: ollamaSetupSkippedPatch(true) });
    setShowOllamaSetup(false);
  }, []);

  return { showOllamaSetup, completeOllamaSetup, skipOllamaSetup } as const;
}
