import { useCallback, useEffect, useRef, useState } from "react";
import type { AgentMessage, CloneMode } from "@/types/agent";

export interface PendingCloneDialog {
  messageId: string;
  canSummarize: boolean;
  error?: string | null;
}

export interface CloneActivationOptions {
  shouldActivateOnComplete?: () => boolean;
}

export type CloneMessageHandler = (
  messageId: string,
  mode: CloneMode,
  customFocus?: string,
  options?: CloneActivationOptions,
) => Promise<void>;

export function useChatClone(
  messages: AgentMessage[],
  onCloneMessage?: CloneMessageHandler,
) {
  const [pendingClone, setPendingClone] = useState<PendingCloneDialog | null>(null);
  const [cloneBusy, setCloneBusy] = useState(false);
  const pendingCloneRef = useRef<PendingCloneDialog | null>(null);
  const runIdRef = useRef(0);

  useEffect(() => {
    pendingCloneRef.current = pendingClone;
  }, [pendingClone]);

  const requestClone = useCallback((id: string) => {
    const index = messages.findIndex((message) => message.id === id);
    const next = {
      messageId: id,
      canSummarize: index >= 0 && index < messages.length - 1,
      error: null,
    };
    pendingCloneRef.current = next;
    setPendingClone(next);
  }, [messages]);

  const submitClone = useCallback(async (mode: CloneMode, customFocus?: string) => {
    if (!pendingClone || !onCloneMessage) return;
    const runId = ++runIdRef.current;
    const messageId = pendingClone.messageId;
    setCloneBusy(true);
    try {
      await onCloneMessage(messageId, mode, customFocus, {
        shouldActivateOnComplete: () =>
          runIdRef.current === runId && pendingCloneRef.current?.messageId === messageId,
      });
      pendingCloneRef.current = null;
      setPendingClone(null);
    } catch {
      setPendingClone((current) => current ? { ...current, error: "failed" } : current);
    } finally {
      if (runIdRef.current === runId) setCloneBusy(false);
    }
  }, [onCloneMessage, pendingClone]);

  const cancelClone = useCallback(() => {
    pendingCloneRef.current = null;
    setPendingClone(null);
    setCloneBusy(false);
  }, []);

  return { pendingClone, cloneBusy, requestClone, submitClone, cancelClone };
}
