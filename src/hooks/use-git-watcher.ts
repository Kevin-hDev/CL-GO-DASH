import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useGitWatcher(projectPath: string | undefined) {
  useEffect(() => {
    if (!projectPath) return;

    void invoke("start_git_watcher", { path: projectPath }).catch(() => {});
    return () => {
      void invoke("stop_git_watcher", { path: projectPath }).catch(() => {});
    };
  }, [projectPath]);
}
