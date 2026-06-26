import { useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { readFile } from "@tauri-apps/plugin-fs";
import { agentStreamManager, type StreamSnapshot } from "./agent-stream-manager";
import { expandToolActivities, expandSegmentsToChat } from "./agent-chat-utils";
import type { AgentMessage, FileAttachment } from "@/types/agent";

const IMAGE_EXTS = ["png", "jpg", "jpeg", "gif", "webp"];
const TEXT_EXTS = [
  "md", "txt", "ts", "tsx", "js", "jsx", "json", "yaml", "yml",
  "toml", "rs", "py", "sh", "css", "html", "xml", "csv", "sql",
  "env", "cfg", "conf", "ini", "log", "svelte", "vue",
];
const MAX_TEXT_CHARS_PER_FILE = 120_000;
const MAX_TEXT_CHARS_PER_MESSAGE = 300_000;

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

function thumbnailToBase64(thumbnail?: string): string | null {
  if (!thumbnail) return null;
  const marker = ";base64,";
  const markerIndex = thumbnail.indexOf(marker);
  if (markerIndex < 0 || !thumbnail.startsWith("data:image/")) return null;
  const payload = thumbnail.slice(markerIndex + marker.length).trim();
  return payload.length > 0 ? payload : null;
}

async function imageAttachmentToBase64(file: FileAttachment): Promise<string | null> {
  if (file.path) {
    try {
      return uint8ToBase64(await readFile(file.path));
    } catch {
      const fallback = thumbnailToBase64(file.thumbnail);
      if (fallback) return fallback;
      console.warn("Lecture image impossible.");
      return null;
    }
  }
  return thumbnailToBase64(file.thumbnail);
}

function clipTextForContext(text: string, maxChars: number): string {
  const chars = Array.from(text);
  if (chars.length <= maxChars) return text;
  return `${chars.slice(0, maxChars).join("")}\n[File truncated: use read_file if full content is needed]`;
}

interface StreamStartState {
  displayMessages: AgentMessage[];
  baseTokenCount: number;
}

export function useAgentStream() {
  const streamingRef = useRef(false);
  const generationRef = useRef<number | null>(null);

  const startStream = useCallback(async (
    sessionId: string,
    model: string,
    provider: string,
    messages: AgentMessage[],
    think: boolean,
    startState: StreamStartState,
    workingDir?: string,
    supportsTools?: boolean,
    supportsThinking?: boolean,
    supportsVision?: boolean,
    reasoningMode?: string | null,
    permissionMode?: string,
    planMode?: boolean,
  ) => {
    streamingRef.current = true;
    await agentStreamManager.startSession(
      sessionId,
      startState.displayMessages,
      startState.baseTokenCount,
    );

    const resolved = await Promise.all(messages.map(async (m) => {
      let images: string[] | null = null;
      let content = m.content;
      let remainingTextChars = MAX_TEXT_CHARS_PER_MESSAGE;

      if (m.files && m.files.length > 0) {
        const imageFiles = m.files.filter((f) => isImageFile(f.name));
        if (imageFiles.length > 0) {
          images = [];
          for (const f of imageFiles) {
            const image = await imageAttachmentToBase64(f);
            if (image) images.push(image);
          }
          if (images.length === 0) images = null;
        }

        const textFiles = m.files.filter((f) => f.path && isTextFile(f.name));
        for (const f of textFiles) {
          try {
            const bytes = await readFile(f.path);
            const text = new TextDecoder().decode(bytes);
            const allowed = Math.min(MAX_TEXT_CHARS_PER_FILE, remainingTextChars);
            if (allowed <= 0) {
              content += `\n\n--- Fichier: ${f.name} ---\n[File omitted: text attachment budget reached]`;
              continue;
            }
            const clipped = clipTextForContext(text, allowed);
            remainingTextChars = Math.max(0, remainingTextChars - allowed);
            content += `\n\n--- Fichier: ${f.name} ---\n${clipped}`;
          } catch {
            console.warn("Lecture fichier texte impossible.");
          }
        }
      }

      return { message: m, content, images };
    }));

    const chatMessages = resolved.flatMap(({ message: m, content, images }) => {
      if (m.role === "assistant" && m.segments && m.segments.length > 0 && !m.tool_calls) {
        return expandSegmentsToChat(m.segments, content);
      }
      if (m.role === "assistant" && m.tool_activities && m.tool_activities.length > 0 && !m.tool_calls) {
        return expandToolActivities(m.tool_activities, content);
      }
      return [{
        role: m.role, content, images,
        tool_calls: m.tool_calls ?? null, tool_name: m.tool_name ?? null,
      }];
    });

    try {
      const gen = await invoke<number>("chat_stream", {
        sessionId,
        model,
        provider,
        messages: chatMessages,
        tools: [],
        think,
        workingDir: workingDir ?? null,
        supportsTools: supportsTools ?? null,
        supportsThinking: supportsThinking ?? null,
        supportsVision: supportsVision ?? null,
        reasoningMode: reasoningMode ?? null,
        permissionMode: permissionMode ?? null,
        planMode: planMode ?? null,
      });
      generationRef.current = gen;
    } catch {
      agentStreamManager.failSession(sessionId);
      streamingRef.current = false;
    }
  }, []);

  const stopStream = useCallback(async (sessionId: string) => {
    const gen = generationRef.current;
    generationRef.current = null;
    await invoke("cancel_agent_request", { sessionId, generation: gen });
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
