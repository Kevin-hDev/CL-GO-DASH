import { act, fireEvent, render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { SessionSummaryGitSection, type SessionSummaryGitState } from "../session-summary-git-section";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: Record<string, unknown>) => {
      const text: Record<string, string> = {
        "agentLocal.sessionSummary.branch": "Branch",
        "agentLocal.sessionSummary.git.save": "Save changes",
        "agentLocal.sessionSummary.git.push": "Send branch",
        "agentLocal.sessionSummary.git.commitTitle": "Save changes",
        "agentLocal.sessionSummary.git.commitDescription": "Describe the work",
        "agentLocal.sessionSummary.git.cancel": "Cancel",
        "agentLocal.sessionSummary.git.confirmCommit": "Save",
        "agentLocal.sessionSummary.git.pushError": "Unable to send the branch",
      };
      if (key.endsWith(".toggle")) return `Branch: ${typeof opts?.branch === "string" ? opts.branch : ""}`;
      if (key.endsWith(".changesToSave")) return `${numberValue(opts?.count)} files to save`;
      if (key.endsWith(".commitsToPush")) return `${numberValue(opts?.count)} commits ready`;
      return text[key] ?? key;
    },
  }),
}));

vi.mock("@/hooks/use-github-branch-auth", () => ({
  useGithubBranchAuth: () => ({ open: false, state: "idle", request: vi.fn() }),
}));

const baseGit: SessionSummaryGitState = {
  isGitRepo: true,
  isLoading: false,
  currentBranch: "main",
  dirtyCount: 0,
  hasRemote: true,
  isGithubRemote: true,
  hasUpstream: true,
  aheadCount: 0,
  behindCount: 0,
  worktrees: [],
  listDirtyFiles: vi.fn().mockResolvedValue([]),
  commit: vi.fn().mockResolvedValue({ ok: true }),
  push: vi.fn().mockResolvedValue({ ok: true }),
  refresh: vi.fn(),
};

describe("SessionSummaryGitSection", () => {
  it("deplie la branche et enregistre les modifications", async () => {
    const commit = vi.fn().mockResolvedValue({ ok: true });
    const listDirtyFiles = vi.fn().mockResolvedValue([{
      path: "src/app.tsx", status: "modified", additions: 3, deletions: 1,
    }]);
    const { getByRole } = render(
      <SessionSummaryGitSection git={{ ...baseGit, dirtyCount: 1, commit, listDirtyFiles }} />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));
    fireEvent.click(getByRole("button", { name: "Save changes" }));
    await act(async () => {});
    fireEvent.click(getByRole("button", { name: "Save" }));

    await act(async () => {});
    expect(commit).toHaveBeenCalledWith(undefined);
  });

  it("affiche le push seulement quand des commits sont prets", async () => {
    const push = vi.fn().mockResolvedValue({ ok: true });
    const { getByRole, getByText } = render(
      <SessionSummaryGitSection git={{ ...baseGit, aheadCount: 2, push }} />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));
    expect(getByText("2 commits ready")).toBeTruthy();
    fireEvent.click(getByRole("button", { name: "Send branch" }));

    await act(async () => {});
    expect(push).toHaveBeenCalledOnce();
  });

  it("n'ouvre pas GitHub pour une erreur d'authentification d'un autre depot", async () => {
    const push = vi.fn().mockResolvedValue({ ok: false, kind: "authentication_required" });
    const { getByRole, getByText } = render(
      <SessionSummaryGitSection
        git={{ ...baseGit, isGithubRemote: false, aheadCount: 1, push }}
      />,
    );

    fireEvent.click(getByRole("button", { name: "Branch: main" }));
    fireEvent.click(getByRole("button", { name: "Send branch" }));
    await act(async () => {});

    expect(getByText("Unable to send the branch")).toBeTruthy();
  });
});

function numberValue(value: unknown) {
  return typeof value === "number" || typeof value === "string" ? value : "";
}
