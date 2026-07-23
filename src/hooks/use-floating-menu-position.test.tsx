import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { useFloatingMenuPosition } from "./use-floating-menu-position";

function FloatingFixture() {
  const { anchorRef, floatingRef, floatingStyle } =
    useFloatingMenuPosition(true, "right", 6, "auto");
  return (
    <>
      <button ref={(node) => { anchorRef.current = node; }} data-anchor>
        anchor
      </button>
      <div ref={floatingRef} style={floatingStyle}>menu</div>
    </>
  );
}

afterEach(() => {
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("useFloatingMenuPosition", () => {
  it("keeps a large menu inside a small webview", async () => {
    vi.spyOn(HTMLElement.prototype, "getBoundingClientRect")
      .mockImplementation(function getRect(this: HTMLElement) {
        if (this.dataset.anchor !== undefined) {
          return {
            x: 250, y: 8, top: 8, right: 280, bottom: 36, left: 250,
            width: 30, height: 28, toJSON: () => ({}),
          };
        }
        return {
          x: 0, y: 0, top: 0, right: 340, bottom: 420, left: 0,
          width: 340, height: 420, toJSON: () => ({}),
        };
      });
    vi.spyOn(HTMLElement.prototype, "offsetWidth", "get").mockReturnValue(340);
    vi.spyOn(HTMLElement.prototype, "offsetHeight", "get").mockReturnValue(420);
    vi.stubGlobal("innerWidth", 300);
    vi.stubGlobal("innerHeight", 240);

    render(<FloatingFixture />);

    await waitFor(() => expect(screen.getByText("menu")).toHaveStyle({
      top: "42px",
      left: "12px",
      maxWidth: "276px",
      maxHeight: "186px",
      visibility: "visible",
    }));
  });
});
