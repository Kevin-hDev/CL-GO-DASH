import { describe, expect, it, vi } from "vitest";
import { render, fireEvent, waitFor } from "@testing-library/react";
import { BranchSelector } from "../branch-selector";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({ t: (key: string) => key }),
}));

vi.mock("@/hooks/use-keyboard", () => ({ useKeyboard: () => {} }));
vi.mock("@/hooks/use-click-outside", () => ({ useClickOutside: () => {} }));

function git(overrides = {}) {
  return {
    branches: [
      { name: "main", is_current: true, is_remote: false, dirty_count: 0 },
      { name: "feature/manual", is_current: false, is_remote: false, dirty_count: 0 },
    ],
    worktrees: [],
    currentBranch: "main",
    dirtyCount: 0,
    isGitRepo: true,
    isLoading: false,
    refresh: vi.fn(),
    checkout: vi.fn().mockResolvedValue({ ok: true }),
    create: vi.fn().mockResolvedValue({ ok: true }),
    ...overrides,
  };
}

function renderSelector(overrides = {}, props = {}) {
  return render(
    <BranchSelector
      git={git(overrides)}
      locked={false}
      onConflict={vi.fn()}
      onWorktreeSelect={vi.fn()}
      onGithubAuthRequired={vi.fn()}
      {...props}
    />,
  );
}

describe("BranchSelector clone branch linking", () => {
  it("notifies parent after selecting a branch manually", async () => {
    const checkout = vi.fn().mockResolvedValue({ ok: true });
    const onBranchReady = vi.fn().mockResolvedValue(undefined);
    const { container } = renderSelector({ checkout }, { onBranchReady });

    fireEvent.click(container.querySelector(".bs-btn") as HTMLElement);
    const item = Array.from(container.querySelectorAll(".bs-item")).find((el) =>
      el.textContent?.includes("feature/manual"),
    ) as HTMLElement;
    fireEvent.click(item);

    await waitFor(() => expect(checkout).toHaveBeenCalledWith("feature/manual"));
    await waitFor(() => expect(onBranchReady).toHaveBeenCalledWith("feature/manual"));
  });

  it("notifies parent after creating a branch manually", async () => {
    const create = vi.fn().mockResolvedValue({ ok: true });
    const onBranchReady = vi.fn().mockResolvedValue(undefined);
    const { container } = renderSelector({ create }, { onBranchReady });

    fireEvent.click(container.querySelector(".bs-btn") as HTMLElement);
    const items = Array.from(container.querySelectorAll(".bs-item"));
    fireEvent.click(items[items.length - 1]);
    fireEvent.change(container.querySelector(".bs-create-input") as HTMLInputElement, {
      target: { value: "feature/manual" },
    });
    fireEvent.keyDown(container.querySelector(".bs-create-input") as HTMLInputElement, { key: "Enter" });

    await waitFor(() => expect(create).toHaveBeenCalledWith("feature/manual"));
    await waitFor(() => expect(onBranchReady).toHaveBeenCalledWith("feature/manual"));
  });
});
