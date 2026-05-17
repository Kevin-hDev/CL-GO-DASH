import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ProjectSelector } from "./project-selector";
import { BranchSelector } from "./branch-selector";
import { BranchConflictDialog } from "./branch-conflict-dialog";
import type { useGitBranch } from "@/hooks/use-git-branch";
import type { useSessionProject } from "@/hooks/use-session-project";
import type { Project } from "@/types/agent";

interface ChatProjectControlsProps {
  projects: Project[];
  projectState: ReturnType<typeof useSessionProject>;
  git: ReturnType<typeof useGitBranch>;
  onWorktreeSelect: (path: string, branch: string) => void;
}

export function ChatProjectControls({
  projects,
  projectState,
  git,
  onWorktreeSelect,
}: ChatProjectControlsProps) {
  const [branchConflict, setBranchConflict] = useState<{ branch: string; dirtyCount: number } | null>(null);

  return (
    <>
      <div style={{ display: "flex", alignItems: "center", gap: "var(--space-xs)", flexWrap: "wrap" }}>
        <ProjectSelector
          projects={projects}
          selectedProjectId={projectState.selectedProjectId}
          locked={projectState.locked}
          hidden={projectState.hidden}
          onSelect={projectState.setSelectedProjectId}
          onAddProject={() => void projectState.handleAddProject()}
        />
        <BranchSelector
          git={git}
          locked={false}
          onConflict={(branch, dirtyCount) => setBranchConflict({ branch, dirtyCount })}
          onWorktreeSelect={onWorktreeSelect}
        />
      </div>

      {branchConflict && projectState.selectedProject && (
        <BranchConflictDialog
          targetBranch={branchConflict.branch}
          dirtyCount={branchConflict.dirtyCount}
          projectPath={projectState.selectedProject.path}
          onCancel={() => setBranchConflict(null)}
          onCommitAndSwitch={(branch) => {
            void (async () => {
              try {
                await invoke("commit_and_checkout_git_branch", {
                  path: projectState.selectedProject!.path,
                  branchName: branch,
                });
                await git.refresh();
              } catch (e) {
                console.error("commit_and_checkout:", e);
              }
              setBranchConflict(null);
            })();
          }}
        />
      )}
    </>
  );
}
