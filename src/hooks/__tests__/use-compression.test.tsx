import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useCompression } from "../use-compression";

let streamHandler: ((event: { payload: unknown }) => void) | null = null;

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn((_event: string, handler: (event: { payload: unknown }) => void) => {
    streamHandler = handler;
    return Promise.resolve(() => {});
  }),
}));

function emit(sessionId: string, event: string, data: Record<string, unknown> = {}) {
  act(() => {
    streamHandler?.({ payload: { sessionId, event: { event, data } } });
  });
}

beforeEach(() => {
  streamHandler = null;
});

describe("useCompression", () => {
  it("arrête l'indicateur quand le stream échoue", () => {
    const { result } = renderHook(() => useCompression("s1"));

    emit("s1", "compressing", { status: "start" });
    expect(result.current.isCompressing).toBe(true);

    emit("s1", "error");
    expect(result.current.isCompressing).toBe(false);
  });

  it("arrête l'indicateur quand la compression est terminée", () => {
    const { result } = renderHook(() => useCompression("s1"));

    emit("s1", "compressing", { status: "start" });
    emit("s1", "compressionComplete");

    expect(result.current.isCompressing).toBe(false);
  });

  it("réinitialise l'indicateur quand la session change", () => {
    const { result, rerender } = renderHook(
      ({ sessionId }) => useCompression(sessionId),
      { initialProps: { sessionId: "s1" } },
    );

    emit("s1", "compressing", { status: "start" });
    expect(result.current.isCompressing).toBe(true);

    rerender({ sessionId: "s2" });

    expect(result.current.isCompressing).toBe(false);
  });
});
