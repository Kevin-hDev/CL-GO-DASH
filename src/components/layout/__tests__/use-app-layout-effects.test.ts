import { fireEvent, renderHook, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  shouldAutoHideSidebarForAgentPanels,
  sidebarHiddenOffsetFromWidth,
  useAgentPanelsAutoSidebar,
  useAppLayoutShortcuts,
  useSidebarHiddenOffset,
} from "../use-app-layout-effects";

class ResizeObserverMock {
  static instances: ResizeObserverMock[] = [];
  readonly observe = vi.fn();
  readonly disconnect = vi.fn();

  constructor(private readonly callback: ResizeObserverCallback) {
    ResizeObserverMock.instances.push(this);
  }

  trigger() {
    this.callback([], this as unknown as ResizeObserver);
  }
}

const originalResizeObserver = globalThis.ResizeObserver;
const originalRequestAnimationFrame = window.requestAnimationFrame;
const originalCancelAnimationFrame = window.cancelAnimationFrame;

function setElementWidth(element: Element, width: number) {
  element.getBoundingClientRect = () => ({
    width,
    height: 0,
    top: 0,
    right: width,
    bottom: 0,
    left: 0,
    x: 0,
    y: 0,
    toJSON: () => ({}),
  });
}

function installLayoutDom(detailWidth: number) {
  document.body.innerHTML = `
    <div class="app-sidebar-block"></div>
    <section class="app-detail-panel">
      <div class="agent-detail-with-preview">
        <aside class="fp-panel open"></aside>
        <aside class="ft-panel open"></aside>
      </div>
    </section>
  `;
  const detail = document.querySelector(".app-detail-panel")!;
  setElementWidth(detail, detailWidth);
  return detail;
}

beforeEach(() => {
  ResizeObserverMock.instances = [];
  globalThis.ResizeObserver = ResizeObserverMock as unknown as typeof ResizeObserver;
  window.requestAnimationFrame = (callback: FrameRequestCallback) => {
    callback(0);
    return 1;
  };
  window.cancelAnimationFrame = () => {};
});

afterEach(() => {
  document.body.innerHTML = "";
  globalThis.ResizeObserver = originalResizeObserver;
  window.requestAnimationFrame = originalRequestAnimationFrame;
  window.cancelAnimationFrame = originalCancelAnimationFrame;
  vi.restoreAllMocks();
});

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

describe("useAgentPanelsAutoSidebar", () => {
  it("masque quand les panneaux agent sont trop serres", async () => {
    installLayoutDom(760);
    const setSidebarOpen = vi.fn();

    renderHook(() => useAgentPanelsAutoSidebar(true, setSidebarOpen));

    await waitFor(() => expect(setSidebarOpen).toHaveBeenCalledWith(false));
  });

  it("ne rouvre pas automatiquement la sidebar apres un masquage automatique", () => {
    installLayoutDom(900);
    const setSidebarOpen = vi.fn();

    renderHook(() => useAgentPanelsAutoSidebar(false, setSidebarOpen));
    ResizeObserverMock.instances[0]?.trigger();

    expect(setSidebarOpen).not.toHaveBeenCalled();
  });
});

describe("useSidebarHiddenOffset", () => {
  it("mesure la vraie largeur de la sidebar pour le masquage", async () => {
    installLayoutDom(900);
    const sidebar = document.querySelector(".app-sidebar-block")!;
    setElementWidth(sidebar, 318);

    const { result } = renderHook(() => useSidebarHiddenOffset(true));

    await waitFor(() => expect(result.current).toBe(sidebarHiddenOffsetFromWidth(318)));
  });

  it("actualise l'offset quand la largeur change pendant le masquage", async () => {
    installLayoutDom(900);
    const sidebar = document.querySelector(".app-sidebar-block")!;
    setElementWidth(sidebar, 318);
    const { result, rerender } = renderHook(
      ({ open }) => useSidebarHiddenOffset(open),
      { initialProps: { open: true } },
    );
    await waitFor(() => expect(result.current).toBe(sidebarHiddenOffsetFromWidth(318)));

    setElementWidth(sidebar, 120);
    rerender({ open: false });
    const lastObserver = ResizeObserverMock.instances[ResizeObserverMock.instances.length - 1];
    lastObserver?.trigger();

    expect(result.current).toBe(sidebarHiddenOffsetFromWidth(120));
  });
});
