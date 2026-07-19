import { fireEvent, render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SessionSummaryCommits } from "../session-summary-commits";
import type { SessionSummaryGitState } from "../session-summary-git-types";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    i18n: { language: "fr" },
    t: (key: string) => ({
      "agentLocal.sessionSummary.commits.toggle": "Afficher les commits",
      "agentLocal.sessionSummary.commits.title": "Commits",
      "agentLocal.sessionSummary.commits.empty": "Aucun commit",
      "agentLocal.sessionSummary.commits.error": "Erreur",
      "agentLocal.sessionSummary.commits.noMessage": "Sans message",
      "agentLocal.sessionSummary.commits.noFiles": "Aucun fichier",
      "common.loading": "Chargement",
    } as Record<string, string>)[key] ?? key,
  }),
}));

vi.mock("@/components/ui/icons", () => ({
  CaretDown: () => <span />,
  CaretLeft: () => <span />,
  Hash: () => <span />,
}));

vi.mock("@/components/file-preview/file-icon", () => ({
  FileIcon: () => <span />,
}));

describe("SessionSummaryCommits", () => {
  it("ouvre un commit puis transmet le fichier choisi", async () => {
    const commit = {
      id: "a".repeat(40),
      short_id: "aaaaaaaa",
      message: "Premier commit",
      timestamp: 1_700_000_000,
    };
    const file = {
      path: "src/app.ts",
      status: "modified" as const,
      additions: 3,
      deletions: 1,
    };
    const git = gitState({
      listCommits: vi.fn().mockResolvedValue({ commits: [commit] }),
      listCommitFiles: vi.fn().mockResolvedValue([file]),
    });
    const onOpenFile = vi.fn();
    const view = render(<SessionSummaryCommits git={git} onOpenFile={onOpenFile} />);

    fireEvent.click(view.getByRole("button", { name: "Afficher les commits" }));
    await waitFor(() => expect(view.getByText("Premier commit")).toBeTruthy());
    fireEvent.click(view.getByRole("button", { name: /Premier commit/ }));
    await waitFor(() => expect(view.getByText("src/app.ts")).toBeTruthy());
    fireEvent.click(view.getByRole("button", { name: /src\/app.ts/ }));

    expect(onOpenFile).toHaveBeenCalledWith(commit, file);
  });
});

function gitState(overrides: Partial<SessionSummaryGitState>): SessionSummaryGitState {
  return {
    repositoryPath: "/repo",
    isGitRepo: true,
    isLoading: false,
    currentBranch: "main",
    branches: [],
    worktrees: [],
    dirtyCount: 0,
    hasRemote: false,
    isGithubRemote: false,
    hasRemoteBranch: false,
    aheadCount: 0,
    behindCount: 0,
    listDirtyFiles: vi.fn().mockResolvedValue([]),
    listCommits: vi.fn().mockResolvedValue({ commits: [] }),
    listCommitFiles: vi.fn().mockResolvedValue([]),
    commit: vi.fn().mockResolvedValue({ ok: true }),
    push: vi.fn().mockResolvedValue({ ok: true }),
    previewBranchMerge: vi.fn(),
    mergeBranch: vi.fn().mockResolvedValue({ ok: true }),
    refresh: vi.fn(),
    ...overrides,
  };
}
