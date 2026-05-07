import { describe, it, expect } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useChatScroll } from "../use-chat-scroll";

describe("useChatScroll", () => {
  it("initialise avec isAtBottom = true", () => {
    const { result } = renderHook(() => useChatScroll());
    expect(result.current.isAtBottom).toBe(true);
  });

  it("setIsAtBottom met à jour le state", () => {
    const { result } = renderHook(() => useChatScroll());
    act(() => {
      result.current.setIsAtBottom(false);
    });
    expect(result.current.isAtBottom).toBe(false);
  });

  it("scrollToBottom remet isAtBottom à true", () => {
    const { result } = renderHook(() => useChatScroll());
    act(() => {
      result.current.setIsAtBottom(false);
    });
    expect(result.current.isAtBottom).toBe(false);
    act(() => {
      result.current.scrollToBottom();
    });
    expect(result.current.isAtBottom).toBe(true);
  });

  it("scrollToBottom ne jette pas si déjà en bas", () => {
    const { result } = renderHook(() => useChatScroll());
    expect(() => {
      act(() => {
        result.current.scrollToBottom();
      });
    }).not.toThrow();
  });
});
