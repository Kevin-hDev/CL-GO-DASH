import { act, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { DEFAULT_APP_NAV } from "@/types/navigation";
import { useAgentLocalControlledPreview } from "../use-agent-local-controlled-preview";
import type { useFilePreview } from "../use-file-preview";

function filePreviewState() {
  return {
    open: true,
    fullscreen: false,
    activeTab: "summary",
    tabs: [],
    width: 360,
    extraWidth: 0,
    resizing: false,
    setOpen: vi.fn(),
    setFullscreen: vi.fn(),
    setExtraWidth: vi.fn(),
    setActiveTab: vi.fn(),
    toggleOpen: vi.fn(),
    closePanel: vi.fn(),
    openOperation: vi.fn(),
    openPath: vi.fn(),
    closeTab: vi.fn(),
    startResize: vi.fn(),
  } as unknown as ReturnType<typeof useFilePreview>;
}

describe("useAgentLocalControlledPreview", () => {
  it("ferme aussi l'arborescence quand la preview se ferme", () => {
    const preview = filePreviewState();
    const onNavChange = vi.fn();

    const { result } = renderHook(() => useAgentLocalControlledPreview({
      navState: {
        ...DEFAULT_APP_NAV.agentLocal,
        previewOpen: true,
        fileTreeOpen: true,
      },
      filePreviewState: preview,
      onNavChange,
    }));

    act(() => result.current.closePanel());

    expect(preview.closePanel).toHaveBeenCalled();
    expect(onNavChange).toHaveBeenCalledWith({
      previewOpen: false,
      previewFullscreen: false,
      fileTreeOpen: false,
    });
  });

  it("ferme aussi l'arborescence quand toggleOpen replie la preview", () => {
    const preview = filePreviewState();
    const onNavChange = vi.fn();

    const { result } = renderHook(() => useAgentLocalControlledPreview({
      navState: {
        ...DEFAULT_APP_NAV.agentLocal,
        previewOpen: true,
        previewFullscreen: true,
        fileTreeOpen: true,
      },
      filePreviewState: preview,
      onNavChange,
    }));

    act(() => result.current.toggleOpen());

    expect(preview.setOpen).toHaveBeenCalledWith(false);
    expect(preview.setFullscreen).toHaveBeenCalledWith(false);
    expect(onNavChange).toHaveBeenCalledWith({
      previewOpen: false,
      previewFullscreen: false,
      previewActiveTab: "summary",
      fileTreeOpen: false,
    });
  });
});
