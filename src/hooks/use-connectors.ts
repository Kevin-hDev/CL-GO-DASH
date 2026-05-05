import { useCallback, useEffect, useState } from "react";
import { homeDir, join } from "@tauri-apps/api/path";
import { readTextFile, writeTextFile } from "@tauri-apps/plugin-fs";
import { IS_MAC } from "@/lib/platform";
import { MCP_CATALOG } from "@/lib/mcp-catalog";
import type { ConfiguredMcp, ConfiguredMcpFull, McpConnectorSpec } from "@/types/mcp";

const FILENAME = "mcp-connectors.json";

async function storagePath(): Promise<string> {
  const home = await homeDir();
  return join(home, ".local", "share", "cl-go-dash", FILENAME);
}

const MAX_CONNECTORS = 32;

const VALID_ID = /^[a-zA-Z0-9_-]+$/;
const ALLOWED_PROGRAMS = ["npx", "uvx", "deno"];
const VALID_ARG = /^[a-zA-Z0-9@/_.:=-]+$/;

function isValidInstallCommand(cmd: string): boolean {
  const parts = cmd.split(/\s+/);
  if (parts.length < 2) return false;
  if (!ALLOWED_PROGRAMS.includes(parts[0])) return false;
  return parts.slice(1).every((p) => VALID_ARG.test(p));
}

function validateConnectors(data: unknown): ConfiguredMcp[] {
  if (!Array.isArray(data)) return [];
  const result: ConfiguredMcp[] = [];
  for (const item of data) {
    if (result.length >= MAX_CONNECTORS) break;
    if (typeof item !== "object" || item === null) continue;
    const r = item as Record<string, unknown>;
    if (typeof r.id !== "string" || typeof r.status !== "string") continue;
    if (!VALID_ID.test(r.id) || r.id.length > 64) continue;
    if (r.status !== "connected" && r.status !== "disconnected") continue;
    const cmd = typeof r.install_command === "string" ? r.install_command : undefined;
    if (cmd && !isValidInstallCommand(cmd)) continue;
    result.push({
      id: r.id,
      status: r.status,
      enabled_in_chat: r.enabled_in_chat === true,
      endpoint: typeof r.endpoint === "string" ? r.endpoint : undefined,
      install_command: cmd,
      env_keys: Array.isArray(r.env_keys) ? r.env_keys.filter((k: unknown) => typeof k === "string").slice(0, 10) : undefined,
    });
  }
  return result;
}

async function loadConfigured(): Promise<ConfiguredMcp[]> {
  try {
    const path = await storagePath();
    const raw = await readTextFile(path);
    return validateConnectors(JSON.parse(raw));
  } catch {
    return [];
  }
}

async function saveConfigured(list: ConfiguredMcp[]): Promise<void> {
  try {
    const path = await storagePath();
    await writeTextFile(path, JSON.stringify(list));
  } catch {
    console.warn("[mcp-connectors] save failed");
  }
}

export function useConnectors() {
  const catalog: McpConnectorSpec[] = MCP_CATALOG.filter(
    (c) => !c.os_restrict || (c.os_restrict === "macos" && IS_MAC),
  );

  const [items, setItems] = useState<ConfiguredMcp[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    void loadConfigured().then((data) => { setItems(data); setLoading(false); });
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
    const spec = catalog.find((s) => s.id === id);
    await persist([...items, {
      id, status: "connected", enabled_in_chat: true,
      endpoint: spec?.endpoint,
      install_command: spec?.install_command,
      env_keys: spec?.env_keys,
    }]);
  }, [items, persist, catalog]);

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
