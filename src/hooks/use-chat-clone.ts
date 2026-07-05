import { useCallback, useState } from "react";
import type { AgentMessage, CloneMode } from "@/types/agent";

export interface PendingCloneDialog {
  messageId: string;
  canSummarize: boolean;
  error?: string | null;
}

export function useChatClone(
  messages: AgentMessage[],
  onCloneMessage?: (messageId: string, mode: CloneMode, customFocus?: string) => Promise<void>,
) {
  const [pendingClone, setPendingClone] = useState<PendingCloneDialog | null>(null);
  const [cloneBusy, setCloneBusy] = useState(false);

  const requestClone = useCallback((id: string) => {
    const index = messages.findIndex((message) => message.id === id);
    setPendingClone({
      messageId: id,
      canSummarize: index >= 0 && index < messages.length - 1,
      error: null,
    });
  }, [messages]);

  const submitClone = useCallback(async (mode: CloneMode, customFocus?: string) => {
    if (!pendingClone || !onCloneMessage) return;
    setCloneBusy(true);
    try {
      await onCloneMessage(pendingClone.messageId, mode, customFocus);
      setPendingClone(null);
    } catch {
      setPendingClone((current) => current ? { ...current, error: "failed" } : current);
    } finally {
      setCloneBusy(false);
    }
  }, [onCloneMessage, pendingClone]);

  const cancelClone = useCallback(() => setPendingClone(null), []);

  return { pendingClone, cloneBusy, requestClone, submitClone, cancelClone };
}
