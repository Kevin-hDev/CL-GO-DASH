import { describe, expect, it, vi } from "vitest";
import { act, fireEvent, render } from "@testing-library/react";
import { SessionSummaryBubble } from "../session-summary-bubble";
import type { SessionSummaryHookState } from "@/hooks/use-session-summary";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const text: Record<string, string> = {
        "agentLocal.sessionSummary.tooltip": "Toggle summary",
        "agentLocal.sessionSummary.title": "Session summary",
        "agentLocal.sessionSummary.environment": "Environment",
        "agentLocal.sessionSummary.modifications": "Changes",
        "agentLocal.sessionSummary.branch": "Branch",
        "agentLocal.sessionSummary.noGit": "No Git repository",
        "agentLocal.sessionSummary.noChanges": "No changes",
        "agentLocal.sessionSummary.sections.todos": "Todo list",
        "agentLocal.sessionSummary.sections.plans": "Plan",
        "agentLocal.sessionSummary.sections.subagents": "Subagents",
        "agentLocal.sessionSummary.emptyTodos": "No active todo",
        "agentLocal.sessionSummary.emptyPlans": "No plan",
        "agentLocal.sessionSummary.emptySubagents": "No subagent",
        "agentLocal.sessionSummary.planStatus.awaiting_approval": "to approve",
        "agentLocal.sessionSummary.subagentType.explorer": "Explore",
        "todos.status.completed": "completed",
        "todos.status.pending": "pending",
        "subagents.completed": "completed",
        "common.loading": "Loading",
      };
      if (key === "agentLocal.sessionSummary.todoProgress") {
        const done = typeof opts?.done === "number" || typeof opts?.done === "string" ? opts.done : "";
        const total = typeof opts?.total === "number" || typeof opts?.total === "string" ? opts.total : "";
        return `${done}/${total}`;
      }
      return text[key] ?? key;
    },
  }),
}));

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span data-testid="caret" />,
  ClipboardText: () => <span data-testid="clipboard" />,
  FilePlus: () => <span data-testid="file-plus" />,
  FileText: () => <span data-testid="file-text" />,
  GitBranch: () => <span data-testid="git-branch" />,
  Users: () => <span data-testid="users" />,
}));

const git = {
  isGitRepo: true,
  isLoading: false,
  currentBranch: "main",
  worktrees: [],
};

describe("SessionSummaryBubble", () => {
  it("ouvre la bulle et affiche modifications + branche", () => {
    const { getByRole, getByText } = render(
      <SessionSummaryBubble summary={summary()} git={git} />,
    );

    fireEvent.click(getByRole("button", { name: "Toggle summary" }));

    expect(getByRole("dialog", { name: "Session summary" })).toBeTruthy();
    expect(getByText("Environment")).toBeTruthy();
    expect(getByText("+3")).toBeTruthy();
    expect(getByText("-1")).toBeTruthy();
    expect(getByText("main")).toBeTruthy();
  });

  it("déplie les sections Todo, Plan et Sous-agents", () => {
    const { getByRole, getByText } = render(
      <SessionSummaryBubble summary={summary()} git={git} />,
    );

    fireEvent.click(getByRole("button", { name: "Toggle summary" }));
    fireEvent.click(getByRole("button", { name: "Todo list (1)" }));
    fireEvent.click(getByRole("button", { name: "Plan (1)" }));
    fireEvent.click(getByRole("button", { name: "Subagents (1)" }));

    expect(getByText("Implement UI")).toBeTruthy();
    expect(getByText("1/2")).toBeTruthy();
    expect(getByText("Plan title")).toBeTruthy();
    expect(getByText("Explore · Explorer")).toBeTruthy();
  });

  it("déplie une todo list et affiche ses tâches", () => {
    const { getByRole, getByText } = render(
      <SessionSummaryBubble summary={summary()} git={git} />,
    );

    fireEvent.click(getByRole("button", { name: "Toggle summary" }));
    fireEvent.click(getByRole("button", { name: "Todo list (1)" }));
    fireEvent.click(getByRole("button", { name: /Implement UI/ }));

    expect(getByText("One")).toBeTruthy();
    expect(getByText("Two")).toBeTruthy();
    expect(getByText("pending")).toBeTruthy();
  });

  it("ouvre le plan au clic sur son entrée", () => {
    const onOpenPlan = vi.fn();
    const { getByRole, getByText } = render(
      <SessionSummaryBubble summary={summary()} git={git} onOpenPlan={onOpenPlan} />,
    );

    fireEvent.click(getByRole("button", { name: "Toggle summary" }));
    fireEvent.click(getByRole("button", { name: "Plan (1)" }));
    fireEvent.click(getByText("Plan title"));

    expect(onOpenPlan).toHaveBeenCalledWith(expect.objectContaining({ id: "plan-1" }));
  });

  it("ouvre la conversation enfant au clic sur un sous-agent", () => {
    const onOpenSubagent = vi.fn();
    const { getByRole, getByText } = render(
      <SessionSummaryBubble summary={summary()} git={git} onOpenSubagent={onOpenSubagent} />,
    );

    fireEvent.click(getByRole("button", { name: "Toggle summary" }));
    fireEvent.click(getByRole("button", { name: "Subagents (1)" }));
    fireEvent.click(getByText("Explore · Explorer"));

    expect(onOpenSubagent).toHaveBeenCalledWith("child-1");
  });

  it("affiche un fallback sans dépôt Git", () => {
    const { getByRole, getByText } = render(
      <SessionSummaryBubble
        summary={summary({ additions: 0, deletions: 0 })}
        git={{ ...git, isGitRepo: false, currentBranch: "" }}
      />,
    );

    fireEvent.click(getByRole("button", { name: "Toggle summary" }));

    expect(getByText("No changes")).toBeTruthy();
    expect(getByText("No Git repository")).toBeTruthy();
  });

  it("affiche le fallback branche si git est absent", () => {
    const { getByRole, getByText } = render(
      <SessionSummaryBubble summary={summary()} />,
    );

    fireEvent.click(getByRole("button", { name: "Toggle summary" }));

    expect(getByText("No Git repository")).toBeTruthy();
  });

  it("ferme la bulle avec Échap sans propager l'événement", () => {
    const { getByRole, queryByRole } = render(
      <SessionSummaryBubble summary={summary()} git={git} />,
    );
    fireEvent.click(getByRole("button", { name: "Toggle summary" }));
    expect(getByRole("dialog", { name: "Session summary" })).toBeTruthy();

    const globalShortcut = vi.fn();
    window.addEventListener("keydown", globalShortcut);
    const event = new KeyboardEvent("keydown", { key: "Escape", bubbles: true, cancelable: true });
    act(() => {
      window.dispatchEvent(event);
    });
    window.removeEventListener("keydown", globalShortcut);

    expect(event.defaultPrevented).toBe(true);
    expect(globalShortcut).not.toHaveBeenCalled();
    expect(queryByRole("dialog", { name: "Session summary" })).toBeNull();
  });
});

function summary(changes = { additions: 3, deletions: 1 }): SessionSummaryHookState {
  return {
    session: null,
    changes: { ...changes, files: changes.additions + changes.deletions > 0 ? 1 : 0 },
    activeTodos: [{
      id: "todo-1",
      title: "Implement UI",
      status: "active",
      created_at: "",
      updated_at: "",
      todos: [
        { content: "One", status: "completed" },
        { content: "Two", status: "pending" },
      ],
    }],
    plans: [{
      id: "plan-1",
      title: "Plan title",
      status: "awaiting_approval",
      path: "/tmp/plan.md",
      created_at: "",
      updated_at: "",
    }],
    subagents: [{
      sessionId: "child-1",
      name: "Explorer",
      type: "explorer",
      status: "completed",
      promptPreview: "",
    }],
  };
}
