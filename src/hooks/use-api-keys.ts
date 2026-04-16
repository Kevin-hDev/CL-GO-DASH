import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { ProviderSpec } from "@/types/api";

/**
 * Hook de gestion des clés API.
 * - Charge le catalogue complet (LLM + Search) depuis le backend
 * - Charge la liste des providers configurés
 * - Expose set / delete / test / refresh
 *
 * IMPORTANT : aucune clé n'est jamais retournée côté JS — uniquement les ids.
 */
export function useApiKeys() {
  const [catalog, setCatalog] = useState<ProviderSpec[]>([]);
  const [configuredIds, setConfiguredIds] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);

  const loadCatalog = useCallback(async () => {
    const [llm, search] = await Promise.all([
      invoke<ProviderSpec[]>("list_llm_providers_catalog"),
      invoke<ProviderSpec[]>("list_search_providers_catalog"),
    ]);
    setCatalog([...llm, ...search]);
  }, []);

  const loadConfigured = useCallback(async () => {
    const ids = await invoke<string[]>("list_configured_providers");
    setConfiguredIds(ids);
  }, []);

  const refresh = useCallback(async () => {
    await Promise.all([loadCatalog(), loadConfigured()]);
    setLoading(false);
  }, [loadCatalog, loadConfigured]);

  useEffect(() => {
    refresh().catch(() => setLoading(false));
  }, [refresh]);

  const setKey = useCallback(
    async (provider: string, key: string) => {
      await invoke("set_api_key", { provider, key });
      await loadConfigured();
    },
    [loadConfigured],
  );

  const deleteKey = useCallback(
    async (provider: string) => {
      await invoke("delete_api_key", { provider });
      await loadConfigured();
    },
    [loadConfigured],
  );

  const testKey = useCallback(async (provider: string): Promise<void> => {
    // Préfère le test riche (catalog-aware) s'il existe, sinon fallback sur test_api_key
    const spec = catalog.find((p) => p.id === provider);
    if (spec?.category === "llm") {
      await invoke("test_llm_connection", { providerId: provider });
    } else if (spec?.category === "search" || spec?.category === "scraping") {
      await invoke("test_search_connection", { providerId: provider });
    } else {
      await invoke("test_api_key", { provider });
    }
  }, [catalog]);

  return {
    catalog,
    configuredIds,
    configured: catalog.filter((p) => configuredIds.includes(p.id)),
    loading,
    refresh,
    setKey,
    deleteKey,
    testKey,
  };
}
