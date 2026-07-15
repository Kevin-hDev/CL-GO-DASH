/* @vitest-environment jsdom */
import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { AgentSidePanel } from "../agent-side-panel";

describe("AgentSidePanel", () => {
  afterEach(() => cleanup());

  it("place la poignée avant le panneau sans lui réserver de largeur", () => {
    const view = render(
      <AgentSidePanel
        open
        fullscreen={false}
        displayWidth={600}
        fullscreenWidth={1000}
        fullscreenSwitching={false}
        resizing={false}
        mode="browser"
        onResizeStart={vi.fn()}
        previewContent={null}
      />,
    );

    const panel = view.container.querySelector(".asp-panel");
    const slot = view.container.querySelector(".asp-resize-slot");
    const handle = view.container.querySelector(".asp-resize");

    expect(slot).not.toBeNull();
    expect(slot?.nextElementSibling).toBe(panel);
    expect(handle?.parentElement).toBe(slot);
  });
});
