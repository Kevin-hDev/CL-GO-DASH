import { useCallback, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { MCP_CATALOG } from "@/lib/mcp-catalog";
import { connectorPayload } from "@/lib/mcp-connector-payload";
import { cleanupTauriListener } from "@/lib/tauri-listen";

export type GithubBranchAuthState = "idle" | "connecting" | "testing" | "error";

const GITHUB_CONNECTOR = MCP_CATALOG.find((c) => c.id === "github");

export function useGithubBranchAuth(onConnected?: () => void) {
  const [open, setOpen] = useState(false);
  const [state, setState] = useState<GithubBranchAuthState>("idle");
  const unlistenRef = useRef<Promise<() => void> | null>(null);

  const cleanup = useCallback(() => {
    if (unlistenRef.current) cleanupTauriListener(unlistenRef.current);
    unlistenRef.current = null;
  }, []);

  const request = useCallback(() => {
    setState("idle");
    setOpen(true);
  }, []);

  const cancel = useCallback(() => {
    cleanup();
    void invoke("cancel_mcp_oauth", { connectorId: "github" }).catch(() => {});
    setOpen(false);
    setState("idle");
  }, [cleanup]);

  const connect = useCallback(async () => {
    if (!GITHUB_CONNECTOR?.endpoint) {
      setState("error");
      return;
    }
    cleanup();
    setState("connecting");

    const handleOAuthResult = async (payload: Record<string, unknown>) => {
      if (payload.connector_id !== "github") return;
      cleanup();
      if (payload.success !== true) {
        setState("error");
        return;
      }
      try {
        setState("testing");
        const connector = connectorPayload(GITHUB_CONNECTOR);
        await invoke("test_mcp_connector", { connector });
        await invoke("add_mcp_connector", { connector });
        onConnected?.();
        setOpen(false);
        setState("idle");
      } catch {
        await invoke("delete_mcp_oauth_token", { connectorId: "github" }).catch(() => {});
        setState("error");
      }
    };

    const unlistenPromise = listen<Record<string, unknown>>("mcp-oauth-result", (event) => {
      void handleOAuthResult(event.payload);
    });
    unlistenRef.current = unlistenPromise;

    try {
      await unlistenPromise;
      await invoke("start_mcp_oauth", {
        connectorId: "github",
        endpoint: GITHUB_CONNECTOR.endpoint,
      });
    } catch {
      cleanup();
      setState("error");
    }
  }, [cleanup, onConnected]);

  return { open, state, request, cancel, connect };
}
