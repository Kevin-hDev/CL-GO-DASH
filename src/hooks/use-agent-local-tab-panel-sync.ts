import { useEffect, useRef, type SetStateAction } from "react";
import type { FilePreviewActiveTab } from "@/types/file-preview";
import type { AgentLocalNavState, DeepPartial } from "@/types/navigation";

interface FilePreviewPanelState {
  open: boolean;
  activeTab: FilePreviewActiveTab;
  fullscreen: boolean;
  setOpen: (value: SetStateAction<boolean>) => void;
  setActiveTab: (value: SetStateAction<FilePreviewActiveTab>) => void;
  setFullscreen: (value: SetStateAction<boolean>) => void;
}

interface TerminalPanelState {
  isOpen: boolean;
  activeTabId: string | null;
  togglePanel: () => void;
  setActiveTab: (id: string) => void;
}

interface PanelSyncOpts {
  navState: AgentLocalNavState;
  filePreview: FilePreviewPanelState;
  terminal: TerminalPanelState;
  onNavChange?: (partial: DeepPartial<AgentLocalNavState>) => void;
  onNavReplace?: (partial: DeepPartial<AgentLocalNavState>) => void;
}

export function useAgentLocalTabPanelSync({
  navState,
  filePreview,
  terminal,
  onNavChange,
  onNavReplace,
}: PanelSyncOpts) {
  const {
    open: actualPreviewOpen,
    activeTab: actualPreviewActiveTab,
    fullscreen: actualPreviewFullscreen,
    setOpen,
    setActiveTab,
    setFullscreen,
  } = filePreview;
  const { isOpen: actualTerminalOpen, activeTabId, togglePanel, setActiveTab: setTerminalActiveTab } = terminal;
  const reportedPanelState = useRef(false);
  const restoredPanelNavKey = useRef<string | null>(null);
  const skipStaleReport = useRef(false);
  const panelNavKey = JSON.stringify([
    navState.previewOpen,
    navState.previewActiveTab,
    navState.previewFullscreen,
    navState.terminalOpen,
    navState.terminalActiveTabId,
  ]);

  useEffect(() => {
    if (restoredPanelNavKey.current === panelNavKey) return;
    restoredPanelNavKey.current = panelNavKey;
    let restored = false;

    if (actualPreviewOpen !== navState.previewOpen) {
      restored = true;
      setOpen(navState.previewOpen);
    }
    if (actualPreviewActiveTab !== navState.previewActiveTab) {
      restored = true;
      setActiveTab(navState.previewActiveTab);
    }
    if (actualPreviewFullscreen !== navState.previewFullscreen) {
      restored = true;
      setFullscreen(navState.previewFullscreen);
    }
    if (actualTerminalOpen !== navState.terminalOpen) {
      restored = true;
      togglePanel();
    }
    if (navState.terminalActiveTabId && activeTabId !== navState.terminalActiveTabId) {
      restored = true;
      setTerminalActiveTab(navState.terminalActiveTabId);
    }

    skipStaleReport.current = restored;
  }, [
    panelNavKey,
    navState.previewOpen, navState.previewActiveTab, navState.previewFullscreen,
    navState.terminalOpen, navState.terminalActiveTabId,
    actualPreviewOpen, actualPreviewActiveTab, actualPreviewFullscreen,
    setOpen, setActiveTab, setFullscreen,
    actualTerminalOpen, activeTabId, togglePanel, setTerminalActiveTab,
  ]);

  useEffect(() => {
    if (skipStaleReport.current) {
      skipStaleReport.current = false;
      return;
    }
    const report = reportedPanelState.current ? onNavChange : onNavReplace ?? onNavChange;
    reportedPanelState.current = true;
    report?.({
      previewOpen: actualPreviewOpen,
      previewActiveTab: actualPreviewActiveTab,
      previewFullscreen: actualPreviewFullscreen,
      terminalOpen: actualTerminalOpen,
      terminalActiveTabId: activeTabId ?? null,
    });
  }, [
    actualPreviewOpen, actualPreviewActiveTab, actualPreviewFullscreen,
    actualTerminalOpen, activeTabId, onNavChange, onNavReplace,
  ]);
}
