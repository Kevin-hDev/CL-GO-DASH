import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Plus } from "../icons";

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
