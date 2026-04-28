import { useMemo } from "react";
import { collectFileOperations } from "@/lib/file-preview-utils";
import type { AgentMessage } from "@/types/agent";

export function useSessionFiles(messages: AgentMessage[]) {
  return useMemo(() => collectFileOperations(messages), [messages]);
}
