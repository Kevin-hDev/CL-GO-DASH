import { useCallback, type RefObject } from "react";
import { persistAgentMessage, type AgentSendPayload } from "./agent-message-send";
import type { AgentMessage } from "@/types/agent";

interface Params {
  sessionId: string | null;
  messages: AgentMessage[];
  permissionModeRef: RefObject<string | undefined>;
  savingRef: RefObject<boolean>;
  runOrDefer: (workingDir: string | undefined, run: () => Promise<void>) => Promise<void>;
  doStream: Parameters<typeof persistAgentMessage>[0]["doStream"];
}

export function useAgentMessageSend(params: Params) {
  const {
    sessionId, messages, permissionModeRef, savingRef, runOrDefer, doStream,
  } = params;
  const persist = useCallback(async (payload: AgentSendPayload) => {
    if (!sessionId) return;
    while (savingRef.current) await new Promise((resolve) => setTimeout(resolve, 50));
    await persistAgentMessage({
      ...payload,
      sessionId,
      messages,
      permissionMode: permissionModeRef.current,
      doStream,
    });
  }, [doStream, messages, permissionModeRef, savingRef, sessionId]);

  return useCallback(async (
    text: string,
    sentFiles?: AgentSendPayload["sentFiles"],
    workingDir?: string,
    projectId?: string,
    skills?: AgentSendPayload["skills"],
  ) => {
    if (!text.trim() && !sentFiles?.length && !skills?.length) return;
    const payload = { text, sentFiles, workingDir, projectId, skills };
    await runOrDefer(workingDir, () => persist(payload));
  }, [persist, runOrDefer]);
}
