import { cleanup, render } from "@testing-library/react";
import type { ReactNode } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { SubagentAccordion } from "../subagent-accordion";
import type { SubagentInfo } from "@/types/agent";

afterEach(() => cleanup());

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      if (key === "subagents.backgroundCount") {
        const count = typeof opts?.count === "number" || typeof opts?.count === "string"
          ? opts.count
          : "";
        return `${count} sous-agents actifs`;
      }
      if (key === "subagents.running") return "en cours...";
      if (key === "subagents.stopAll") return "Tout arrêter";
      if (key === "subagents.stop") return "Arrêter";
      if (key === "subagents.open") return "Ouvrir";
      return key;
    },
  }),
}));

vi.mock("@/components/ui/tooltip", () => ({
  Tooltip: ({ children }: { children: ReactNode }) => <>{children}</>,
}));

describe("SubagentAccordion", () => {
  it("affiche les noms produit et le statut sur la première ligne", () => {
    const { container, getByText } = render(
      <SubagentAccordion
        subagents={[agent("coder", "Audit subagents long"), agent("explorer", "Audit web")]}
        onCancel={vi.fn()}
        onOpen={vi.fn()}
      />,
    );

    expect(getByText("Claudiator")).toBeTruthy();
    expect(getByText("Geminitor")).toBeTruthy();
    expect(getByText("Audit subagents long")).toBeTruthy();
    expect(container.querySelectorAll(".sa-agent-heading .sa-agent-status")).toHaveLength(2);
  });
});

function agent(type: "explorer" | "coder", name: string): SubagentInfo {
  return {
    sessionId: `${type}-child`,
    name,
    type,
    status: "running",
    promptPreview: "",
  };
}
