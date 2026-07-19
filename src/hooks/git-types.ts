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
  repositoryPath: string;
  branches: BranchInfo[];
  worktrees: WorktreeInfo[];
  currentBranch: string;
  dirtyCount: number;
  hasRemote: boolean;
  isGithubRemote: boolean;
  hasRemoteBranch: boolean;
  aheadCount: number;
  behindCount: number;
  isGitRepo: boolean;
  isLoading: boolean;
}

export interface GitDirtyFile {
  path: string;
  previous_path?: string | null;
  status: string;
  additions: number;
  deletions: number;
}

export interface GitCommitSummary {
  id: string;
  short_id: string;
  message: string;
  timestamp: number;
}

export interface GitCommitPage {
  commits: GitCommitSummary[];
  next_cursor?: string;
}

export interface GitCommitFile {
  path: string;
  previous_path?: string | null;
  status: "added" | "deleted" | "renamed" | "copied" | "modified" | "changed";
  additions: number;
  deletions: number;
}

export interface GitUncommittedSnapshot {
  head_commit: string;
  files: GitDirtyFile[];
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

export interface BranchMergePreview {
  source_branch: string;
  target_branch: string;
  commits: number;
  dirty_files: GitDirtyFile[];
}

export type GitDeleteMode = "clean" | "discard" | "preserve";

export interface GitPushTarget {
  repositoryPath: string;
  branch: string;
}

export type GitActionResult =
  | { ok: true }
  | { ok: false; kind: GitActionErrorKind };

export type GitActionErrorKind =
  | "no_remote"
  | "authentication_required"
  | "permission_denied"
  | "remote_changed"
  | "network_unavailable"
  | "context_changed"
  | "branch_unavailable"
  | "dirty_worktree"
  | "nothing_to_merge"
  | "merge_conflict"
  | "internal_error";
