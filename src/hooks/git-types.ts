export interface BranchInfo {
  name: string;
  is_current: boolean;
  is_remote: boolean;
  dirty_count: number;
}

export interface WorktreeInfo {
  path: string;
  branch: string;
  is_current: boolean;
}

export interface GitBranchState {
  branches: BranchInfo[];
  worktrees: WorktreeInfo[];
  currentBranch: string;
  dirtyCount: number;
  hasRemote: boolean;
  isGithubRemote: boolean;
  hasUpstream: boolean;
  aheadCount: number;
  behindCount: number;
  isGitRepo: boolean;
  isLoading: boolean;
}

export interface GitDirtyFile {
  path: string;
  status: string;
  additions: number;
  deletions: number;
}

export interface BranchDeletePreview {
  branch: string;
  is_current: boolean;
  fallback_branch?: string;
  dirty_files: GitDirtyFile[];
  unmerged_commits: number;
}

export interface WorktreeDeletePreview {
  path: string;
  branch: string;
  dirty_files: GitDirtyFile[];
}

export type GitDeleteMode = "clean" | "discard" | "preserve";

export type GitActionResult =
  | { ok: true }
  | { ok: false; kind: GitActionErrorKind };

export type GitActionErrorKind =
  | "no_remote"
  | "authentication_required"
  | "remote_changed"
  | "internal_error";
