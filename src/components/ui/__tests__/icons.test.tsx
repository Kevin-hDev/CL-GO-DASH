import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { SidebarToggleIcon } from "@/components/layout/toolbar-icons";
import { Plus } from "../icons";
import { TerminalSquare } from "../lucide-icons";

afterEach(() => cleanup());

describe("safe Phosphor icons", () => {
  it("applies CSS variable sizes via style instead of svg attributes", () => {
    const { container } = render(<Plus size="var(--icon-sm)" />);
    const svg = container.querySelector("svg");

    expect(svg).toHaveStyle({ width: "var(--icon-sm)", height: "var(--icon-sm)" });
    expect(svg?.getAttribute("width")).not.toBe("var(--icon-sm)");
    expect(svg?.getAttribute("height")).not.toBe("var(--icon-sm)");
  });
});

describe("safe SVG icon sizes", () => {
  it("applies CSS variables via style for Lucide icons", () => {
    const { container } = render(<TerminalSquare size="var(--chrome-icon-md)" />);
    const svg = container.querySelector("svg");

    expect(svg).toHaveStyle({ width: "var(--chrome-icon-md)", height: "var(--chrome-icon-md)" });
    expect(svg?.getAttribute("width")).not.toBe("var(--chrome-icon-md)");
    expect(svg?.getAttribute("height")).not.toBe("var(--chrome-icon-md)");
  });

  it("applies CSS variables via style for local toolbar icons", () => {
    const { container } = render(<SidebarToggleIcon size="var(--chrome-icon-md)" />);
    const svg = container.querySelector("svg");

    expect(svg).toHaveStyle({ width: "var(--chrome-icon-md)", height: "var(--chrome-icon-md)" });
    expect(svg?.getAttribute("width")).not.toBe("var(--chrome-icon-md)");
    expect(svg?.getAttribute("height")).not.toBe("var(--chrome-icon-md)");
  });
});
