import { renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DEFAULT_APP_NAV } from "@/types/navigation";
import { useAgentLocalTabPanelSync } from "../use-agent-local-tab-panel-sync";
import type { FilePreviewActiveTab } from "@/types/file-preview";

function preview(open: boolean, activeTab: FilePreviewActiveTab = "summary") {
  return {
    open,
    activeTab,
    fullscreen: false,
    setOpen: vi.fn(),
    setActiveTab: vi.fn(),
    setFullscreen: vi.fn(),
  };
}

function terminal(open = false) {
  return {
    isOpen: open,
    activeTabId: null,
    togglePanel: vi.fn(),
    setActiveTab: vi.fn(),
  };
}

describe("useAgentLocalTabPanelSync", () => {
  it("ne republie pas un état preview obsolète pendant une restauration", () => {
    const filePreview = preview(true);
    const terminalPanel = terminal();
    const onNavChange = vi.fn();
    const onNavReplace = vi.fn();

    const { rerender } = renderHook(
      ({ open }) => useAgentLocalTabPanelSync({
        navState: DEFAULT_APP_NAV.agentLocal,
        filePreview: { ...filePreview, open },
        terminal: terminalPanel,
        onNavChange,
        onNavReplace,
      }),
      { initialProps: { open: true } },
    );

    expect(filePreview.setOpen).toHaveBeenCalledWith(false);
    expect(onNavChange).not.toHaveBeenCalled();
    expect(onNavReplace).not.toHaveBeenCalled();

    rerender({ open: false });

    expect(onNavReplace).toHaveBeenCalledWith({
      previewOpen: false,
      previewActiveTab: "summary",
      previewFullscreen: false,
      terminalOpen: false,
      terminalActiveTabId: null,
    });
  });

  it("publie une ouverture locale sans la refermer", () => {
    const filePreview = preview(false);
    const terminalPanel = terminal();
    const onNavChange = vi.fn();
    const onNavReplace = vi.fn();

    const { rerender } = renderHook(
      ({ open }) => useAgentLocalTabPanelSync({
        navState: DEFAULT_APP_NAV.agentLocal,
        filePreview: { ...filePreview, open },
        terminal: terminalPanel,
        onNavChange,
        onNavReplace,
      }),
      { initialProps: { open: false } },
    );

    onNavChange.mockClear();
    onNavReplace.mockClear();
    rerender({ open: true });

    expect(filePreview.setOpen).not.toHaveBeenCalledWith(false);
    expect(onNavChange).toHaveBeenCalledWith({
      previewOpen: true,
      previewActiveTab: "summary",
      previewFullscreen: false,
      terminalOpen: false,
      terminalActiveTabId: null,
    });
  });
});
