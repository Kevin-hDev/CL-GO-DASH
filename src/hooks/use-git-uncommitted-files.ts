import { useCallback, useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";
import { uncommittedFileOperations } from "@/lib/git-file-preview";
import type { GitUncommittedSnapshot } from "@/hooks/git-types";
import type { FileOperation } from "@/types/file-preview";

interface GitUncommittedSource {
  isGitRepo: boolean;
  currentBranch: string;
  dirtyCount: number;
  listUncommittedFiles: () => Promise<GitUncommittedSnapshot>;
}

export function useGitUncommittedFiles(git: GitUncommittedSource): FileOperation[] {
  const { currentBranch, dirtyCount, isGitRepo, listUncommittedFiles } = git;
  const [state, setState] = useState<{ branch: string; operations: FileOperation[] }>({
    branch: "",
    operations: [],
  });
  const requestRef = useRef(0);
  const load = useCallback(async () => {
    const request = ++requestRef.current;
    try {
      const snapshot = await listUncommittedFiles();
      if (request !== requestRef.current) return;
      setState({
        branch: currentBranch,
        operations: uncommittedFileOperations(snapshot, currentBranch),
      });
    } catch {
      if (request !== requestRef.current) return;
      setState({ branch: currentBranch, operations: [] });
    }
  }, [currentBranch, listUncommittedFiles]);

  useEffect(() => {
    if (!isGitRepo || !currentBranch || dirtyCount === 0) return;
    // eslint-disable-next-line react-hooks/set-state-in-effect -- fetch puis mise à jour asynchrone
    void load();
  }, [currentBranch, dirtyCount, isGitRepo, load]);

  useEffect(() => {
    const unlisten = listen("git-branch-changed", () => void load());
    return () => cleanupTauriListener(unlisten);
  }, [load]);

  if (!isGitRepo || dirtyCount === 0 || state.branch !== currentBranch) return [];
  return state.operations;
}
