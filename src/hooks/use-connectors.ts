import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { IS_MAC } from "@/lib/platform";
import { MCP_CATALOG } from "@/lib/mcp-catalog";
import type { ConfiguredMcp, ConfiguredMcpFull, McpConnectorSpec } from "@/types/mcp";

const STORAGE_KEY = "mcp-connectors.json";

async function loadConfigured(): Promise<ConfiguredMcp[]> {
  try {
    const raw = await invoke<string>("read_text_file", { filename: STORAGE_KEY });
    return JSON.parse(raw) as ConfiguredMcp[];
  } catch {
    return [];
  }
}

async function saveConfigured(list: ConfiguredMcp[]): Promise<void> {
  await invoke("write_text_file", {
    filename: STORAGE_KEY,
    content: JSON.stringify(list),
  });
}

export function useConnectors() {
  const catalog: McpConnectorSpec[] = MCP_CATALOG.filter(
    (c) => !c.os_restrict || (c.os_restrict === "macos" && IS_MAC),
  );

  const [items, setItems] = useState<ConfiguredMcp[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadConfigured().then((data) => { setItems(data); setLoading(false); });
  }, []);

  const persist = useCallback(async (next: ConfiguredMcp[]) => {
    setItems(next);
    await saveConfigured(next);
  }, []);

  const configuredIds = items.map((c) => c.id);

  const configured: ConfiguredMcpFull[] = items
    .map((c) => {
      const spec = catalog.find((s) => s.id === c.id);
      if (!spec) return null;
      return { ...spec, ...c };
    })
    .filter((x): x is ConfiguredMcpFull => x !== null);

  const addConnector = useCallback(async (id: string) => {
    if (items.some((c) => c.id === id)) return;
    await persist([...items, { id, status: "connected", enabled_in_chat: true }]);
  }, [items, persist]);

  const removeConnector = useCallback(async (id: string) => {
    await persist(items.filter((c) => c.id !== id));
  }, [items, persist]);

  const toggleStatus = useCallback(async (id: string) => {
    await persist(items.map((c) =>
      c.id === id
        ? { ...c, status: c.status === "connected" ? "disconnected" as const : "connected" as const }
        : c,
    ));
  }, [items, persist]);

  const toggleChatEnabled = useCallback(async (id: string) => {
    await persist(items.map((c) =>
      c.id === id ? { ...c, enabled_in_chat: !c.enabled_in_chat } : c,
    ));
  }, [items, persist]);

  const isConnected = useCallback(
    (id: string) => items.some((c) => c.id === id && c.status === "connected"),
    [items],
  );

  return {
    catalog, configured, configuredIds, loading,
    addConnector, removeConnector, toggleStatus, toggleChatEnabled, isConnected,
  };
}
