import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useChatScroll } from "../use-chat-scroll";

const makeScrollEl = (scrollTop = 0, scrollHeight = 1000, clientHeight = 500) => ({
  scrollTop,
  scrollHeight,
  clientHeight,
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
});

describe("useChatScroll", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("initialise avec isAtBottom = true", () => {
    const { result } = renderHook(() =>
      useChatScroll({
        messagesLength: 0,
        currentContent: "",
        currentThinking: "",
        currentToolsLength: 0,
      }),
    );
    expect(result.current.isAtBottom).toBe(true);
  });

  it("retourne scrollRef et bottomRef non nuls", () => {
    const { result } = renderHook(() =>
      useChatScroll({
        messagesLength: 0,
        currentContent: "",
        currentThinking: "",
        currentToolsLength: 0,
      }),
    );
    expect(result.current.scrollRef).toBeDefined();
    expect(result.current.bottomRef).toBeDefined();
  });

  it("handleScroll met isAtBottom à false quand loin du bas", () => {
    const { result } = renderHook(() =>
      useChatScroll({
        messagesLength: 0,
        currentContent: "",
        currentThinking: "",
        currentToolsLength: 0,
      }),
    );

    const el = makeScrollEl(0, 1000, 500);
    Object.defineProperty(result.current.scrollRef, "current", {
      value: el,
      writable: true,
    });

    act(() => {
      result.current.handleScroll();
    });

    expect(result.current.isAtBottom).toBe(false);
  });

  it("handleScroll met isAtBottom à true quand proche du bas (< 80px)", () => {
    const { result } = renderHook(() =>
      useChatScroll({
        messagesLength: 0,
        currentContent: "",
        currentThinking: "",
        currentToolsLength: 0,
      }),
    );

    const el = makeScrollEl(960, 1500, 500);
    Object.defineProperty(result.current.scrollRef, "current", {
      value: el,
      writable: true,
    });

    act(() => {
      result.current.handleScroll();
    });

    expect(result.current.isAtBottom).toBe(true);
  });

  it("scrollToBottom n'échoue pas si scrollRef est null", () => {
    const { result } = renderHook(() =>
      useChatScroll({
        messagesLength: 0,
        currentContent: "",
        currentThinking: "",
        currentToolsLength: 0,
      }),
    );

    expect(() => {
      act(() => {
        result.current.scrollToBottom();
      });
    }).not.toThrow();
  });

  it("scrollToBottom met scrollTop à scrollHeight", () => {
    const { result } = renderHook(() =>
      useChatScroll({
        messagesLength: 0,
        currentContent: "",
        currentThinking: "",
        currentToolsLength: 0,
      }),
    );

    const el = makeScrollEl(0, 1000, 500);
    Object.defineProperty(result.current.scrollRef, "current", {
      value: el,
      writable: true,
    });

    act(() => {
      result.current.scrollToBottom();
    });

    expect(el.scrollTop).toBe(1000);
  });

  it("scrollRef est une ref React (objet avec .current)", () => {
    const { result } = renderHook(() =>
      useChatScroll({
        messagesLength: 0,
        currentContent: "",
        currentThinking: "",
        currentToolsLength: 0,
      }),
    );
    expect(result.current.scrollRef).toHaveProperty("current");
  });
});
