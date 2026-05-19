import { useState, useEffect, useCallback, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";

interface BranchInfo {
  name: string;
  is_current: boolean;
  is_remote: boolean;
  dirty_count: number;
}

interface WorktreeInfo {
  path: string;
  branch: string;
  is_current: boolean;
}

interface GitBranchState {
  branches: BranchInfo[];
  worktrees: WorktreeInfo[];
  currentBranch: string;
  dirtyCount: number;
  isGitRepo: boolean;
  isLoading: boolean;
}

export type GitCreateBranchResult = {
  ok: boolean;
  reason?: "github_auth_required";
};

const INITIAL_STATE: GitBranchState = {
  branches: [],
  worktrees: [],
  currentBranch: "",
  dirtyCount: 0,
  isGitRepo: false,
  isLoading: false,
};

export function useGitBranch(projectPath: string | undefined, sessionId?: string) {
  const [state, setState] = useState<GitBranchState>(INITIAL_STATE);
  const pathRef = useRef(projectPath);
  const mountedRef = useRef(true);

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
      setState((s) => ({ ...s, isGitRepo: false, branches: [], worktrees: [] }));
      return;
    }

    setState((s) => ({ ...s, isLoading: true }));

    try {
      const [branches, context, worktrees] = await Promise.all([
        invoke<BranchInfo[]>("list_git_branches", { path }),
        invoke<{ branch: string; is_detached: boolean; dirty_count: number; is_git_repo: boolean }>(
          "get_git_context", { path },
        ),
        invoke<WorktreeInfo[]>("list_git_worktrees", { path }),
      ]);

      if (!mountedRef.current || path !== pathRef.current) return;

      setState({
        branches,
        worktrees,
        currentBranch: context.branch,
        dirtyCount: context.dirty_count,
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
    if (projectPath) {
      void invoke("start_git_watcher", { path: projectPath }).catch(() => {});
    }
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
      if (String(e).includes("GITHUB_AUTH_REQUIRED")) {
        return { ok: false, reason: "github_auth_required" };
      }
      return { ok: false };
    }
  }, [refresh]);

  return { ...state, refresh, checkout, create };
}
