import { fireEvent, renderHook } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { shouldAutoHideSidebarForAgentPanels, useAppLayoutShortcuts } from "../use-app-layout-effects";

describe("useAppLayoutShortcuts", () => {
  it("ne capte pas Ctrl+Alt+B reserve a la preview", () => {
    const toggleSidebar = vi.fn();

    renderHook(() => useAppLayoutShortcuts({
      onBack: vi.fn(),
      onForward: vi.fn(),
      toggleSearch: vi.fn(),
      toggleSidebar,
    }));

    fireEvent.keyDown(window, { code: "KeyB", ctrlKey: true, altKey: true });

    expect(toggleSidebar).not.toHaveBeenCalled();
  });
});

describe("shouldAutoHideSidebarForAgentPanels", () => {
  it("masque seulement quand preview et arborescence sont ouvertes et trop serrees", () => {
    expect(shouldAutoHideSidebarForAgentPanels(760, true, true)).toBe(true);
    expect(shouldAutoHideSidebarForAgentPanels(780, true, true)).toBe(false);
    expect(shouldAutoHideSidebarForAgentPanels(760, true, false)).toBe(false);
    expect(shouldAutoHideSidebarForAgentPanels(760, false, true)).toBe(false);
  });
});
