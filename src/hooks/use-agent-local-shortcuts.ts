import { useEffect } from "react";

interface AgentLocalShortcutsParams {
  activeSessionId?: string | null;
  terminalOpen: boolean;
  terminalTabsCount: number;
  terminalCwd: string;
  onAddTerminalTab: (cwd: string) => void;
  onToggleTerminal: () => void;
}

export function useAgentLocalShortcuts(params: AgentLocalShortcutsParams) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const onMac = navigator.userAgent.includes("Mac");
      const toggle = onMac
        ? (event.metaKey && event.code === "KeyJ")
        : (event.ctrlKey && event.code === "KeyJ");
      if (!toggle || !params.activeSessionId || isEditableTarget(event.target)) return;
      event.preventDefault();
      if (!params.terminalOpen && params.terminalTabsCount === 0) {
        params.onAddTerminalTab(params.terminalCwd);
      } else {
        params.onToggleTerminal();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [params]);
}

function isEditableTarget(target: EventTarget | null): boolean {
  return target instanceof HTMLInputElement
    || target instanceof HTMLTextAreaElement
    || (target instanceof HTMLElement && target.isContentEditable);
}
