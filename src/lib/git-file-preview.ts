import { fileNameFromPath } from "@/lib/file-preview-utils";
import type {
  GitCommitFile,
  GitCommitSummary,
  GitUncommittedSnapshot,
} from "@/hooks/git-types";
import type { FileOperation, GitFilePreviewSource } from "@/types/file-preview";

export function commitFileOperation(
  commit: GitCommitSummary,
  file: GitCommitFile,
  branch: string,
): FileOperation {
  const useParent = file.status === "deleted";
  return {
    id: `git:${commit.id}:${useParent ? "parent" : "commit"}:${file.path}`,
    path: file.path,
    name: fileNameFromPath(file.path),
    type: "read",
    timestamp: commitTimestamp(commit.timestamp),
    additions: file.additions,
    deletions: file.deletions,
    source: gitSource(commit.id, file.path, branch, useParent),
  };
}

export function uncommittedFileOperations(
  snapshot: GitUncommittedSnapshot,
  branch: string,
): FileOperation[] {
  return snapshot.files.slice(0, 200).map((file) => ({
    id: `git-uncommitted:${branch}:${file.path}`,
    path: file.path,
    name: fileNameFromPath(file.path),
    type: "read" as const,
    timestamp: new Date().toISOString(),
    additions: file.additions,
    deletions: file.deletions,
    source: file.status === "deleted"
      ? gitSource(snapshot.head_commit, file.path, branch, false)
      : undefined,
  }));
}

function gitSource(
  commitId: string,
  filePath: string,
  expectedBranch: string,
  useParent: boolean,
): GitFilePreviewSource {
  return { kind: "git", commitId, filePath, expectedBranch, useParent };
}

function commitTimestamp(seconds: number): string {
  const value = new Date(seconds * 1000);
  return Number.isFinite(value.getTime()) ? value.toISOString() : new Date(0).toISOString();
}
