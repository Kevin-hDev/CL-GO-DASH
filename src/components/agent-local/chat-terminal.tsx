import { TerminalPanel } from "@/components/terminal/terminal-panel";
import type { useTerminal } from "@/hooks/use-terminal";

interface ChatTerminalProps {
  terminal: ReturnType<typeof useTerminal>;
}

export function ChatTerminal({ terminal }: ChatTerminalProps) {
  return (
    <TerminalPanel
      tabs={terminal.tabs}
      activeTabId={terminal.activeTabId}
      allTabs={terminal.allTabs()}
      activeGroupKey={terminal.groupKey}
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
    />
  );
}
