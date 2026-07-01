import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { fireEvent, render, screen } from "@testing-library/react";
import { UserMessage } from "../user-message";

vi.mock("@/lib/linkify", () => ({
  linkifyWithPreviews: (content: string) => ({ text: [content], previews: null }),
}));

vi.mock("@/lib/skill-text", () => ({
  highlightSkillNodes: (nodes: React.ReactNode[]) => nodes,
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      if (key === "agentLocal.showMore") return "Show more";
      if (key === "agentLocal.showLess") return "Show less";
      return key;
    },
  }),
}));

let measuredHeight = 100;
const originalScrollHeight = Object.getOwnPropertyDescriptor(HTMLElement.prototype, "scrollHeight");

beforeEach(() => {
  Object.defineProperty(HTMLElement.prototype, "scrollHeight", {
    configurable: true,
    get: () => measuredHeight,
  });
});

describe("UserMessage", () => {
  it("garde les messages courts sans bouton de dépliage", () => {
    measuredHeight = 100;

    render(<UserMessage content="Message court" />);

    expect(screen.getByText("Message court")).toBeTruthy();
    expect(screen.queryByText("Show more")).toBeNull();
  });

  it("limite un long message et permet de le déplier puis replier", () => {
    measuredHeight = 900;

    const { container } = render(<UserMessage content={"Long message\n".repeat(40)} />);
    const content = container.querySelector(".msg-user-content");

    expect(screen.getByText("Show more")).toBeTruthy();
    expect(content).toHaveStyle({ maxHeight: "434px" });

    fireEvent.click(screen.getByText("Show more"));

    expect(screen.getByText("Show less")).toBeTruthy();
    expect(content).toHaveStyle({ maxHeight: "900px" });

    fireEvent.click(screen.getByText("Show less"));

    expect(screen.getByText("Show more")).toBeTruthy();
    expect(content).toHaveStyle({ maxHeight: "434px" });
  });
});

afterEach(() => {
  measuredHeight = 100;
  if (originalScrollHeight) {
    Object.defineProperty(HTMLElement.prototype, "scrollHeight", originalScrollHeight);
  } else {
    Reflect.deleteProperty(HTMLElement.prototype, "scrollHeight");
  }
});
