import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, fireEvent, waitFor } from "@testing-library/react";
import { BranchSelector } from "../branch-selector";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const translations: Record<string, string> = {
        "branches.search": "Rechercher dans les branches",
        "branches.title": "Branches",
        "branches.worktrees": "Worktrees",
        "branches.noMatch": "Aucune branche trouvée",
        "branches.createNew": "Créer et extraire une nouvelle branche…",
        "branches.detachedHead": "HEAD détaché",
      };
      if (key === "branches.uncommitted" && opts?.count != null) {
        const count = typeof opts.count === "number" || typeof opts.count === "string" ? opts.count : "";
        return `Non validés : ${count} fichier(s)`;
      }
      return translations[key] ?? key;
    },
  }),
}));

vi.mock("@/hooks/use-keyboard", () => ({
  useKeyboard: () => {},
}));

vi.mock("@/hooks/use-click-outside", () => ({
  useClickOutside: () => {},
}));

const baseMockGit = {
  branches: [
    { name: "main", is_current: true, is_remote: false, dirty_count: 3 },
    { name: "feat/login", is_current: false, is_remote: false, dirty_count: 0 },
    { name: "fix/bug-42", is_current: false, is_remote: false, dirty_count: 0 },
  ],
  worktrees: [
    { path: "/tmp/current", branch: "main", is_current: true },
    { path: "/tmp/wt-1", branch: "worktree-agent-abc", is_current: false },
  ],
  currentBranch: "main",
  dirtyCount: 3,
  isGitRepo: true,
  isLoading: false,
  refresh: vi.fn(),
  checkout: vi.fn().mockResolvedValue({ ok: true }),
  create: vi.fn().mockResolvedValue(true),
};

function makeMockGit(overrides = {}) {
  return { ...baseMockGit, ...overrides };
}

function openDropdown(container: HTMLElement) {
  const btn = container.querySelector(".bs-btn") as HTMLElement;
  fireEvent.click(btn);
}

describe("BranchSelector", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders current branch name in the button", () => {
    const { container } = render(
      <BranchSelector git={makeMockGit()} locked={false} onConflict={vi.fn()} onWorktreeSelect={vi.fn()} />,
    );
    const btn = container.querySelector(".bs-btn");
    expect(btn?.textContent).toContain("main");
  });

  it("opens dropdown on click showing all branches", () => {
    const { container } = render(
      <BranchSelector git={makeMockGit()} locked={false} onConflict={vi.fn()} onWorktreeSelect={vi.fn()} />,
    );
    openDropdown(container);
    const dropdown = container.querySelector(".bs-dropdown");
    expect(dropdown).not.toBeNull();
    expect(dropdown?.textContent).toContain("feat/login");
    expect(dropdown?.textContent).toContain("fix/bug-42");
  });

  it("filters branches by search input", () => {
    const { container } = render(
      <BranchSelector git={makeMockGit()} locked={false} onConflict={vi.fn()} onWorktreeSelect={vi.fn()} />,
    );
    openDropdown(container);
    const searchInput = container.querySelector(".bs-dropdown-search") as HTMLInputElement;
    fireEvent.change(searchInput, { target: { value: "feat" } });
    const dropdown = container.querySelector(".bs-dropdown");
    expect(dropdown?.textContent).toContain("feat/login");
    expect(dropdown?.textContent).not.toContain("fix/bug-42");
  });

  it("checks out a branch without selecting a worktree", async () => {
    const checkout = vi.fn().mockResolvedValue({ ok: true });
    const onWorktreeSelect = vi.fn();
    const { container } = render(
      <BranchSelector
        git={makeMockGit({ checkout })}
        locked={false}
        onConflict={vi.fn()}
        onWorktreeSelect={onWorktreeSelect}
      />,
    );
    openDropdown(container);
    const dropdown = container.querySelector(".bs-dropdown") as HTMLElement;
    const item = Array.from(dropdown.querySelectorAll(".bs-item")).find((el) =>
      el.textContent?.includes("feat/login"),
    ) as HTMLElement;

    fireEvent.click(item);

    await waitFor(() => expect(checkout).toHaveBeenCalledWith("feat/login"));
    expect(onWorktreeSelect).not.toHaveBeenCalled();
  });

  it("shows dirty count for current branch", () => {
    const { container } = render(
      <BranchSelector git={makeMockGit()} locked={false} onConflict={vi.fn()} onWorktreeSelect={vi.fn()} />,
    );
    openDropdown(container);
    const detail = container.querySelector(".bs-item-detail");
    expect(detail?.textContent).toContain("Non validés : 3 fichier(s)");
  });

  it("returns null when not a git repo", () => {
    const { container } = render(
      <BranchSelector git={makeMockGit({ isGitRepo: false })} locked={false} onConflict={vi.fn()} onWorktreeSelect={vi.fn()} />,
    );
    expect(container.innerHTML).toBe("");
  });

  it("shows locked indicator without dropdown", () => {
    const { container } = render(
      <BranchSelector git={makeMockGit()} locked={true} onConflict={vi.fn()} onWorktreeSelect={vi.fn()} />,
    );
    const indicator = container.querySelector(".bs-indicator");
    expect(indicator).not.toBeNull();
    expect(indicator?.textContent).toContain("main");
    expect(container.querySelector(".bs-btn")).toBeNull();
    expect(container.querySelector(".bs-dropdown")).toBeNull();
  });

  it("shows worktrees section in dropdown", () => {
    const { container } = render(
      <BranchSelector git={makeMockGit()} locked={false} onConflict={vi.fn()} onWorktreeSelect={vi.fn()} />,
    );
    openDropdown(container);
    const labels = container.querySelectorAll(".bs-section-label");
    const labelTexts = Array.from(labels).map((l) => l.textContent);
    expect(labelTexts).toContain("Worktrees");
    const dropdown = container.querySelector(".bs-dropdown");
    expect(dropdown?.textContent).toContain("worktree-agent-abc");
    expect(dropdown?.textContent).not.toContain("/tmp/current");
  });

  it("notifies parent when selecting a worktree", () => {
    const onWorktreeSelect = vi.fn();
    const { container } = render(
      <BranchSelector git={makeMockGit()} locked={false} onConflict={vi.fn()} onWorktreeSelect={onWorktreeSelect} />,
    );
    openDropdown(container);
    const dropdown = container.querySelector(".bs-dropdown") as HTMLElement;
    const item = Array.from(dropdown.querySelectorAll(".bs-item")).find((el) =>
      el.textContent?.includes("worktree-agent-abc"),
    ) as HTMLElement;

    fireEvent.click(item);

    expect(onWorktreeSelect).toHaveBeenCalledWith("/tmp/wt-1", "worktree-agent-abc");
  });
});
