import type { GitCreateBranchErrorKind } from "@/hooks/git-create-branch-error";
import type { BranchInfo, WorktreeInfo } from "@/hooks/use-git-branch";

export function getVisibleBranchOptions(
  branches: BranchInfo[],
  worktrees: WorktreeInfo[],
  search: string,
) {
  const lowerSearch = search.toLowerCase();
  const otherWorktreeBranches = new Set(
    worktrees.filter((w) => !w.is_current && w.branch).map((w) => w.branch),
  );

  return {
    filteredBranches: branches.filter((b) =>
      b.name.toLowerCase().includes(lowerSearch)
        && (b.is_current || !otherWorktreeBranches.has(b.name)),
    ),
    filteredWorktrees: worktrees.filter((w) =>
      !w.is_current && `${w.branch} ${w.path}`.toLowerCase().includes(lowerSearch),
    ),
  };
}

export function branchCreateErrorKey(kind?: GitCreateBranchErrorKind): string {
  switch (kind) {
    case "invalid_name":
      return "branches.errorInvalidName";
    case "name_too_long":
      return "branches.errorNameTooLong";
    case "already_exists":
      return "branches.errorAlreadyExists";
    case "unborn_head":
      return "branches.errorUnbornHead";
    case "github_auth_required":
      return "branches.errorGithubAuth";
    default:
      return "branches.errorInternal";
  }
}
