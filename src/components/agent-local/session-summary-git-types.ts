import type {
  BranchInfo,
  BranchMergePreview,
  GitActionResult,
  GitCommitFile,
  GitCommitPage,
  GitDirtyFile,
  GitPushTarget,
} from "@/hooks/git-types";

export interface SessionSummaryGitState {
  repositoryPath: string;
  isGitRepo: boolean;
  isLoading: boolean;
  currentBranch: string;
  branches: BranchInfo[];
  dirtyCount: number;
  hasRemote: boolean;
  isGithubRemote: boolean;
  hasRemoteBranch: boolean;
  aheadCount: number;
  behindCount: number;
  worktrees: { branch: string; path: string; is_current: boolean }[];
  listDirtyFiles: () => Promise<GitDirtyFile[]>;
  listCommits: (cursor?: string) => Promise<GitCommitPage>;
  listCommitFiles: (commitId: string) => Promise<GitCommitFile[]>;
  commit: (description?: string) => Promise<GitActionResult>;
  push: (target: GitPushTarget) => Promise<GitActionResult>;
  previewBranchMerge: (
    sourceBranch: string,
    expectedTarget: string,
  ) => Promise<BranchMergePreview>;
  mergeBranch: (
    sourceBranch: string,
    expectedTarget: string,
    commitChanges: boolean,
    commitDescription?: string,
  ) => Promise<GitActionResult>;
  refresh: () => Promise<void>;
}
