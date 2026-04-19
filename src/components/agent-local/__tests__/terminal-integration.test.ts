import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useTerminal } from "@/hooks/use-terminal";

describe("terminal integration - non-regression", () => {
  it("terminal state is independent per hook instance", () => {
    const { result: t1 } = renderHook(() => useTerminal("/project-a"));
    const { result: t2 } = renderHook(() => useTerminal("/project-b"));

    act(() => { t1.current.addTab(); });
    expect(t1.current.tabs).toHaveLength(1);
    expect(t2.current.tabs).toHaveLength(0);
  });

  it("multiple addTab calls create distinct tabs", () => {
    const { result } = renderHook(() => useTerminal("/test"));
    act(() => { result.current.addTab("/a"); });
    act(() => { result.current.addTab("/b"); });
    act(() => { result.current.addTab("/c"); });

    const ids = result.current.tabs.map((t) => t.id);
    const unique = new Set(ids);
    expect(unique.size).toBe(3);
  });

  it("reorder does not lose tabs", () => {
    const { result } = renderHook(() => useTerminal("/test"));
    act(() => { result.current.addTab("/a"); });
    act(() => { result.current.addTab("/b"); });
    act(() => { result.current.addTab("/c"); });

    const before = result.current.tabs.map((t) => t.id).sort();
    act(() => { result.current.reorderTabs(0, 2); });
    const after = result.current.tabs.map((t) => t.id).sort();
    expect(after).toEqual(before);
  });

  it("default panel height is 120px", () => {
    const { result } = renderHook(() => useTerminal("/test"));
    expect(result.current.panelHeight).toBe(120);
  });
});
