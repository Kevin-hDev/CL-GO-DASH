import { useRef, useCallback } from "react";
import { invoke, Channel } from "@tauri-apps/api/core";
import { readFile } from "@tauri-apps/plugin-fs";
import type { StreamEvent, AgentMessage } from "@/types/agent";

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

interface StreamCallbacks {
  onToken: (content: string, tokenCount: number, tps: number) => void;
  onThinking: (content: string) => void;
  onToolCall: (name: string, args: Record<string, unknown>) => void;
  onToolResult: (name: string, content: string, isError: boolean) => void;
  onTurnEnd: () => void;
  onPermissionRequest: (id: string, toolName: string, args: Record<string, unknown>) => void;
  onDone: (evalCount: number, finalTps: number, promptTokens: number) => void;
  onError: (message: string) => void;
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
    callbacks: StreamCallbacks,
    workingDir?: string,
  ) => {
    streamingRef.current = true;

    const channel = new Channel<StreamEvent>();
    channel.onmessage = (event: StreamEvent) => {
      switch (event.event) {
        case "token":
          callbacks.onToken(event.data.content, event.data.tokenCount, event.data.tps);
          break;
        case "thinking":
          callbacks.onThinking(event.data.content);
          break;
        case "toolCall":
          callbacks.onToolCall(event.data.name, event.data.arguments);
          break;
        case "toolResult":
          callbacks.onToolResult(event.data.name, event.data.content, event.data.isError);
          break;
        case "turnEnd":
          callbacks.onTurnEnd();
          break;
        case "permissionRequest":
          callbacks.onPermissionRequest(event.data.id, event.data.toolName, event.data.arguments);
          break;
        case "done":
          callbacks.onDone(event.data.evalCount, event.data.finalTps, event.data.promptTokens);
          streamingRef.current = false;
          break;
        case "error":
          callbacks.onError(event.data.message);
          streamingRef.current = false;
          break;
      }
    };

    const chatMessages = await Promise.all(messages.map(async (m) => {
      let images: string[] | null = null;
      let content = m.content;

      if (m.files && m.files.length > 0) {
        // Images → champ images (base64 pour Ollama vision)
        const imageFiles = m.files.filter((f) => f.path && isImageFile(f.name));
        if (imageFiles.length > 0) {
          images = [];
          for (const f of imageFiles) {
            try {
              const bytes = await readFile(f.path);
              images.push(uint8ToBase64(bytes));
            } catch (e) {
              console.warn("Image read failed:", f.path, e);
            }
          }
          if (images.length === 0) images = null;
        }

        // Fichiers texte → contenu ajouté au message
        const textFiles = m.files.filter((f) => f.path && isTextFile(f.name));
        for (const f of textFiles) {
          try {
            const bytes = await readFile(f.path);
            const text = new TextDecoder().decode(bytes);
            content += `\n\n--- Fichier: ${f.name} ---\n${text}`;
          } catch (e) {
            console.warn("Text file read failed:", f.path, e);
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

    await invoke("chat_stream", {
      sessionId,
      model,
      provider,
      messages: chatMessages,
      tools,
      think,
      workingDir: workingDir ?? null,
      onEvent: channel,
    });
  }, []);

  const stopStream = useCallback(async (sessionId: string) => {
    await invoke("cancel_agent_request", { sessionId });
    streamingRef.current = false;
  }, []);

  return { startStream, stopStream, isStreaming: () => streamingRef.current };
}
