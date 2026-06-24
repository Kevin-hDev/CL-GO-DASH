import type { AgentMessage, FileAttachment } from "@/types/agent";

export interface PendingChatFile {
  name: string;
  path?: string;
  preview?: string;
}

export function pendingFilesToAttachments(files?: PendingChatFile[]): FileAttachment[] {
  return (files ?? []).map((file) => ({
    name: file.name,
    path: file.path ?? "",
    mime_type: "",
    size: 0,
    thumbnail: file.preview,
  }));
}

export function createUserMessage(
  content: string,
  files: FileAttachment[],
  skillNames?: string[],
): AgentMessage {
  return {
    id: crypto.randomUUID(),
    role: "user",
    content,
    files,
    timestamp: new Date().toISOString(),
    skill_names: skillNames,
  };
}

export function createEditedUserMessage(
  original: AgentMessage,
  content: string,
): AgentMessage {
  return createUserMessage(
    content,
    original.files.map((file) => ({ ...file })),
    original.skill_names ? [...original.skill_names] : undefined,
  );
}
