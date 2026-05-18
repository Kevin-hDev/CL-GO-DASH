import { useCallback, useMemo, type SetStateAction } from "react";
import type { useFilePreview } from "@/hooks/use-file-preview";
import type { FileOperation } from "@/types/file-preview";
import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";

function applyAction<T>(current: T, action: SetStateAction<T>): T {
  return typeof action === "function" ? (action as (value: T) => T)(current) : action;
}

interface Args {
  navState: AgentLocalNavState;
  filePreviewState: ReturnType<typeof useFilePreview>;
  onNavChange?: (partial: DeepPartial<AgentLocalNavState>) => void;
}

export function useAgentLocalControlledPreview({ navState, filePreviewState, onNavChange }: Args) {
  const setOpen = useCallback((action: SetStateAction<boolean>) => {
    const next = applyAction(navState.previewOpen, action);
    filePreviewState.setOpen(next);
    onNavChange?.({ previewOpen: next });
  }, [filePreviewState, navState.previewOpen, onNavChange]);

  const setFullscreen = useCallback((action: SetStateAction<boolean>) => {
    const next = applyAction(navState.previewFullscreen, action);
    filePreviewState.setFullscreen(next);
    onNavChange?.({ previewFullscreen: next });
  }, [filePreviewState, navState.previewFullscreen, onNavChange]);

  const setActiveTab = useCallback((action: SetStateAction<string>) => {
    const next = applyAction(navState.previewActiveTab, action);
    filePreviewState.setActiveTab(next);
    onNavChange?.({ previewActiveTab: next });
  }, [filePreviewState, navState.previewActiveTab, onNavChange]);

  const toggleOpen = useCallback(() => {
    const nextOpen = !navState.previewOpen;
    filePreviewState.setOpen(nextOpen);
    if (!nextOpen) filePreviewState.setFullscreen(false);
    onNavChange?.({
      previewOpen: nextOpen,
      previewFullscreen: nextOpen ? navState.previewFullscreen : false,
      previewActiveTab: navState.previewActiveTab || "summary",
    });
  }, [filePreviewState, navState.previewActiveTab, navState.previewFullscreen, navState.previewOpen, onNavChange]);

  const closePanel = useCallback(() => {
    filePreviewState.closePanel();
    onNavChange?.({ previewOpen: false, previewFullscreen: false });
  }, [filePreviewState, onNavChange]);

  const openOperation = useCallback((operation: FileOperation) => {
    const tabId = filePreviewState.openOperation(operation);
    onNavChange?.({ previewOpen: true, previewActiveTab: operation.id });
    return tabId;
  }, [filePreviewState, onNavChange]);

  const openPath = useCallback((path: string) => {
    const tabId = filePreviewState.openPath(path);
    onNavChange?.({ previewOpen: true, previewActiveTab: tabId });
    return tabId;
  }, [filePreviewState, onNavChange]);

  const closeTab = useCallback((id: string) => {
    filePreviewState.closeTab(id);
    if (navState.previewActiveTab === id) onNavChange?.({ previewActiveTab: "summary" });
  }, [filePreviewState, navState.previewActiveTab, onNavChange]);

  return useMemo(() => ({
    ...filePreviewState,
    open: navState.previewOpen,
    fullscreen: navState.previewFullscreen,
    activeTab: navState.previewActiveTab,
    setOpen,
    setFullscreen,
    setActiveTab,
    toggleOpen,
    closePanel,
    openOperation,
    openPath,
    closeTab,
  }), [
    closePanel, closeTab, filePreviewState, navState.previewActiveTab,
    navState.previewFullscreen, navState.previewOpen, openOperation,
    openPath, setActiveTab, setFullscreen, setOpen, toggleOpen,
  ]);
}
