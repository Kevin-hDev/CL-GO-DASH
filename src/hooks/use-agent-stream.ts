import { useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { readFile } from "@tauri-apps/plugin-fs";
import { agentStreamManager, type StreamSnapshot } from "./agent-stream-manager";
import type { AgentMessage } from "@/types/agent";

const IMAGE_EXTS = ["png", "jpg", "jpeg", "gif", "webp"];
const TEXT_EXTS = [
  "md", "txt", "ts", "tsx", "js", "jsx", "json", "yaml", "yml",
  "toml", "rs", "py", "sh", "css", "html", "xml", "csv", "sql",
  "env", "cfg", "conf", "ini", "log", "svelte", "vue",
];

function isImageFile(name: string): boolean {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  return IMAGE_EXTS.includes(ext);
}

function isTextFile(name: string): boolean {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  return TEXT_EXTS.includes(ext);
}

function uint8ToBase64(bytes: Uint8Array): string {
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

interface StreamStartState {
  displayMessages: AgentMessage[];
  baseTokenCount: number;
}

export function useAgentStream() {
  const streamingRef = useRef(false);

  const startStream = useCallback(async (
    sessionId: string,
    model: string,
    provider: string,
    messages: AgentMessage[],
    tools: unknown[],
    think: boolean,
    startState: StreamStartState,
    workingDir?: string,
  ) => {
    streamingRef.current = true;
    agentStreamManager.startSession(
      sessionId,
      startState.displayMessages,
      startState.baseTokenCount,
    );

    const chatMessages = await Promise.all(messages.map(async (m) => {
      let images: string[] | null = null;
      let content = m.content;

      if (m.files && m.files.length > 0) {
        const imageFiles = m.files.filter((f) => f.path && isImageFile(f.name));
        if (imageFiles.length > 0) {
          images = [];
          for (const f of imageFiles) {
            try {
              const bytes = await readFile(f.path);
              images.push(uint8ToBase64(bytes));
            } catch {
              console.warn("Lecture image impossible.");
            }
          }
          if (images.length === 0) images = null;
        }

        const textFiles = m.files.filter((f) => f.path && isTextFile(f.name));
        for (const f of textFiles) {
          try {
            const bytes = await readFile(f.path);
            const text = new TextDecoder().decode(bytes);
            content += `\n\n--- Fichier: ${f.name} ---\n${text}`;
          } catch {
            console.warn("Lecture fichier texte impossible.");
          }
        }
      }

      return {
        role: m.role,
        content,
        images,
        tool_calls: m.tool_calls ?? null,
        tool_name: m.tool_name ?? null,
      };
    }));

    try {
      await invoke("chat_stream", {
        sessionId,
        model,
        provider,
        messages: chatMessages,
        tools,
        think,
        workingDir: workingDir ?? null,
      });
    } catch {
      agentStreamManager.failSession(sessionId);
      streamingRef.current = false;
    }
  }, []);

  const stopStream = useCallback(async (sessionId: string) => {
    await invoke("cancel_agent_request", { sessionId });
    streamingRef.current = false;
    agentStreamManager.stopSession(sessionId);
  }, []);

  const subscribeToStream = useCallback(
    (sessionId: string, listener: (snapshot: StreamSnapshot) => void) =>
      agentStreamManager.subscribe(sessionId, listener),
    [],
  );

  const getStreamSnapshot = useCallback(
    (sessionId: string) => agentStreamManager.getSnapshot(sessionId),
    [],
  );

  return {
    startStream,
    stopStream,
    subscribeToStream,
    getStreamSnapshot,
    isStreaming: (sessionId?: string) =>
      sessionId ? agentStreamManager.isStreaming(sessionId) : streamingRef.current,
  };
}
