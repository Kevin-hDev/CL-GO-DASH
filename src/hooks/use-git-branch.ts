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
import {
  loadGitCore,
  loadGitRemoteStatus,
  loadGitWorktrees,
} from "@/hooks/git-refresh";
import type { GitBranchState } from "@/hooks/git-types";

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
  const refreshIdRef = useRef(0);
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
    const refreshId = ++refreshIdRef.current;
    if (!path) {
      setState(INITIAL_STATE);
      return;
    }

    const isCurrent = () => mountedRef.current
      && refreshId === refreshIdRef.current
      && path === pathRef.current;
    setState((current) => current.repositoryPath === path
      ? { ...current, isLoading: true }
      : { ...INITIAL_STATE, repositoryPath: path, isLoading: true });

    try {
      const { branches, context } = await loadGitCore(path);

      if (!isCurrent()) return;

      setState((current) => ({
        ...current,
        branches,
        currentBranch: context.branch,
        dirtyCount: context.dirty_count,
        isGitRepo: context.is_git_repo,
        isLoading: false,
      }));

      void loadGitWorktrees(path).then((worktrees) => {
        if (isCurrent()) setState((current) => ({ ...current, worktrees }));
      }).catch(() => {});
      void loadGitRemoteStatus(path).then((remote) => {
        if (!isCurrent()) return;
        setState((current) => ({
          ...current,
          hasRemote: remote.has_remote,
          remoteStatusError: false,
          isGithubRemote: remote.is_github,
          hasRemoteBranch: remote.has_remote_branch,
          aheadCount: remote.ahead,
          behindCount: remote.behind,
        }));
      }).catch(() => {
        if (isCurrent()) setState((current) => ({ ...current, remoteStatusError: true }));
      });
    } catch {
      if (!isCurrent()) return;
      setState({ ...INITIAL_STATE, repositoryPath: path });
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
