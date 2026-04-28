import { TerminalPanel } from "@/components/terminal/terminal-panel";
import type { useTerminal } from "@/hooks/use-terminal";

interface ChatTerminalProps {
  terminal: ReturnType<typeof useTerminal>;
  defaultCwd: string;
}

export function ChatTerminal({ terminal, defaultCwd }: ChatTerminalProps) {
  return (
    <TerminalPanel
      tabs={terminal.tabs}
      activeTabId={terminal.activeTabId}
      isOpen={terminal.isOpen}
      panelHeight={terminal.panelHeight}
      onAddTab={terminal.addTab}
      onCloseTab={terminal.closeTab}
      onSelectTab={terminal.setActiveTab}
      onRenameTab={terminal.renameTab}
      onReorderTabs={terminal.reorderTabs}
      onTogglePanel={terminal.togglePanel}
      onPtyReady={terminal.setPtyId}
      onResize={terminal.resizePanel}
      onSetMaxHeight={terminal.setMaxHeight}
      defaultCwd={defaultCwd}
    />
  );
}
