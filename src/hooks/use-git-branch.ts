import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import {
  parseCreateBranchError,
  type GitCreateBranchResult,
} from "@/hooks/git-create-branch-error";
import { useGitMutations } from "@/hooks/use-git-mutations";
import { useGitHistory } from "@/hooks/use-git-history";
import { useGitWatcher } from "@/hooks/use-git-watcher";
import type { BranchInfo, GitBranchState, WorktreeInfo } from "@/hooks/git-types";

export type { BranchInfo, WorktreeInfo } from "@/hooks/git-types";
export {
  parseCreateBranchError,
  type GitCreateBranchErrorKind,
  type GitCreateBranchResult,
} from "@/hooks/git-create-branch-error";

const INITIAL_STATE: GitBranchState = {
  repositoryPath: "",
  branches: [],
  worktrees: [],
  currentBranch: "",
  dirtyCount: 0,
  hasRemote: false,
  remoteStatusError: false,
  isGithubRemote: false,
  hasRemoteBranch: false,
  aheadCount: 0,
  behindCount: 0,
  isGitRepo: false,
  isLoading: false,
};

export function useGitBranch(projectPath: string | undefined, sessionId?: string) {
  const [state, setState] = useState<GitBranchState>(INITIAL_STATE);
  const pathRef = useRef(projectPath);
  const mountedRef = useRef(true);
  useGitWatcher(projectPath);

  useEffect(() => {
    mountedRef.current = true;
    return () => { mountedRef.current = false; };
  }, []);

  useEffect(() => {
    pathRef.current = projectPath;
  }, [projectPath]);

  const refresh = useCallback(async () => {
    const path = pathRef.current;
    if (!path) {
      setState((s) => ({
        ...s,
        repositoryPath: "",
        isGitRepo: false,
        branches: [],
        worktrees: [],
        remoteStatusError: false,
      }));
      return;
    }

    setState((s) => ({ ...s, isLoading: true }));

    try {
      const [branches, context, worktrees, remote] = await Promise.all([
        invoke<BranchInfo[]>("list_git_branches", { path }),
        invoke<{ branch: string; is_detached: boolean; dirty_count: number; is_git_repo: boolean }>(
          "get_git_context", { path },
        ),
        invoke<WorktreeInfo[]>("list_git_worktrees", { path }),
        invoke<{ has_remote: boolean; is_github: boolean; has_remote_branch: boolean; ahead: number; behind: number }>(
          "get_git_remote_status", { path },
        ).then((value) => ({ ...value, status_error: false })).catch(() => ({
          has_remote: false,
          is_github: false,
          has_remote_branch: false,
          ahead: 0,
          behind: 0,
          status_error: true,
        })),
      ]);

      if (!mountedRef.current || path !== pathRef.current) return;

      setState({
        repositoryPath: path,
        branches,
        worktrees,
        currentBranch: context.branch,
        dirtyCount: context.dirty_count,
        hasRemote: remote.has_remote,
        remoteStatusError: remote.status_error,
        isGithubRemote: remote.is_github,
        hasRemoteBranch: remote.has_remote_branch,
        aheadCount: remote.ahead,
        behindCount: remote.behind,
        isGitRepo: context.is_git_repo,
        isLoading: false,
      });
    } catch {
      if (!mountedRef.current || path !== pathRef.current) return;
      setState((s) => ({ ...s, isGitRepo: false, isLoading: false }));
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [projectPath, sessionId, refresh]);

  useEffect(() => {
    const unlisten = listen("git-branch-changed", () => {
      void refresh();
    });

    return () => {
      cleanupTauriListener(unlisten);
    };
  }, [refresh]);

  const checkout = useCallback(async (branchName: string): Promise<{ ok: boolean; dirtyCount?: number }> => {
    const path = pathRef.current;
    if (!path) return { ok: false };

    try {
      await invoke("checkout_git_branch", { path, branchName });
      await refresh();
      return { ok: true };
    } catch (e) {
      const msg = String(e);
      if (msg.includes("DIRTY:")) {
        const match = /DIRTY:(\d+)/.exec(msg);
        const count = match ? parseInt(match[1], 10) : 0;
        return { ok: false, dirtyCount: count };
      }
      return { ok: false };
    }
  }, [refresh]);

  const create = useCallback(async (branchName: string): Promise<GitCreateBranchResult> => {
    const path = pathRef.current;
    if (!path) return { ok: false };

    try {
      await invoke("create_git_branch", { path, branchName });
      await refresh();
      return { ok: true };
    } catch (e) {
      const kind = parseCreateBranchError(e);
      if (kind === "github_auth_required") {
        return { ok: false, reason: "github_auth_required", kind };
      }
      return { ok: false, kind: kind ?? "internal_error" };
    }
  }, [refresh]);

  const mutations = useGitMutations(pathRef, refresh);
  const history = useGitHistory(pathRef, state.currentBranch);

  return { ...state, refresh, checkout, create, ...mutations, ...history };
}
