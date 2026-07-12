import { useCallback, useEffect } from "react";
import { useOllamaConnectionRetry } from "@/hooks/use-ollama-connection-retry";
import type { DroppedFile } from "@/hooks/use-file-drop";
import type { AgentMessage } from "@/types/agent-message";
import type { RetryIndicatorState } from "@/types/agent";
import type { SessionTab } from "@/types/agent-session";

interface Params {
  chat: {
    messages: AgentMessage[];
    reload: (id: string) => void | Promise<void>;
    edit: (id: string, content: string) => void | Promise<void>;
    error?: string;
    isConnectionError?: boolean;
    isStreaming: boolean;
    retryIndicator?: RetryIndicatorState | null;
  };
  permission: {
    family: "chat" | "tools" | null;
    refresh: () => Promise<void>;
  };
  projectPath?: string;
  activeSessionTab?: SessionTab | null;
  onLinkCloneGitBranch?: (
    projectPath: string,
    sessionId: string,
    branchName: string,
  ) => Promise<void>;
  setPreview: (file: DroppedFile) => void;
}

export function useChatViewRuntime(params: Params) {
  const {
    projectPath, activeSessionTab, onLinkCloneGitBranch, setPreview,
  } = params;
  const {
    messages, reload, edit, error, isConnectionError, isStreaming, retryIndicator,
  } = params.chat;
  const { family: permissionFamily, refresh: refreshPermission } = params.permission;
  useEffect(() => {
    if (messages.length > 0 && permissionFamily === null && isStreaming) {
      void refreshPermission();
    }
  }, [
    isStreaming,
    messages.length,
    permissionFamily,
    refreshPermission,
  ]);

  const handleRetry = useCallback(() => {
    const message = [...messages].reverse().find((item) => item.role === "user");
    if (message) void reload(message.id);
  }, [messages, reload]);
  const connectionRetry = useOllamaConnectionRetry({
    error,
    isConnectionError,
    isStreaming,
    onRetry: handleRetry,
  });
  const handleBranchReady = useCallback(async (branchName: string) => {
    if (!projectPath || !activeSessionTab || activeSessionTab.is_main || activeSessionTab.git_branch) return;
    await onLinkCloneGitBranch?.(projectPath, activeSessionTab.session_id, branchName);
  }, [activeSessionTab, onLinkCloneGitBranch, projectPath]);
  const handleReload = useCallback((id: string) => void reload(id), [reload]);
  const handleEdit = useCallback(
    (id: string, content: string) => void edit(id, content),
    [edit],
  );
  const handleFileClick = useCallback((file: { name: string; path?: string; thumbnail?: string }) => {
    setPreview({
      name: file.name,
      path: file.path,
      type: "",
      size: 0,
      preview: file.thumbnail,
    });
  }, [setPreview]);

  return {
    handleBranchReady,
    handleRetry,
    handleReload,
    handleEdit,
    handleFileClick,
    retryIndicator: retryIndicator ?? connectionRetry.indicator,
    showError: !!error && !isStreaming && !connectionRetry.suppressError,
  };
}
