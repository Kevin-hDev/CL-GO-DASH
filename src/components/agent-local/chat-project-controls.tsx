import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { ProjectSelector } from "./project-selector";
import { BranchSelector } from "./branch-selector";
import { BranchConflictDialog } from "./branch-conflict-dialog";
import { BranchGithubAuthDialog } from "./branch-github-auth-dialog";
import { CloneGitBranchButton } from "./clone-git-branch-button";
import { useGithubBranchAuth } from "@/hooks/use-github-branch-auth";
import type { useGitBranch } from "@/hooks/use-git-branch";
import type { useSessionProject } from "@/hooks/use-session-project";
import type { Project } from "@/types/agent";

interface ChatProjectControlsProps {
  projects: Project[];
  projectState: ReturnType<typeof useSessionProject>;
  git: ReturnType<typeof useGitBranch>;
  onWorktreeSelect: (path: string, branch: string) => void;
  cloneGitBranch?: {
    visible: boolean;
    state: "idle" | "loading" | "success" | "linked";
    label: string;
    disabled?: boolean;
    branchSelectorLockedLabel?: string;
    onCreate: () => void;
  };
}

export function ChatProjectControls({
  projects,
  projectState,
  git,
  onWorktreeSelect,
  cloneGitBranch,
}: ChatProjectControlsProps) {
  const { t } = useTranslation();
  const githubAuth = useGithubBranchAuth(() => void git.refresh());
  const [branchConflict, setBranchConflict] = useState<{
    branch: string;
    dirtyCount: number;
    busy?: boolean;
    error?: string;
  } | null>(null);

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
          locked={!!cloneGitBranch?.branchSelectorLockedLabel}
          lockedLabel={cloneGitBranch?.branchSelectorLockedLabel}
          onConflict={(branch, dirtyCount) => setBranchConflict({ branch, dirtyCount })}
          onWorktreeSelect={onWorktreeSelect}
          onGithubAuthRequired={githubAuth.request}
        />
        {cloneGitBranch?.visible && (
          <CloneGitBranchButton
            state={cloneGitBranch.state}
            label={cloneGitBranch.label}
            disabled={cloneGitBranch.disabled}
            onClick={cloneGitBranch.onCreate}
          />
        )}
      </div>

      {branchConflict && projectState.selectedProject && (
        <BranchConflictDialog
          targetBranch={branchConflict.branch}
          dirtyCount={branchConflict.dirtyCount}
          projectPath={projectState.selectedProject.path}
          busy={branchConflict.busy}
          error={branchConflict.error}
          onCancel={() => setBranchConflict(null)}
          onCommitAndSwitch={(branch, commitDescription) => {
            void (async () => {
              setBranchConflict((current) => current ? { ...current, busy: true, error: undefined } : current);
              try {
                await invoke("commit_and_checkout_git_branch", {
                  path: projectState.selectedProject!.path,
                  branchName: branch,
                  commitDescription,
                });
                await git.refresh();
                setBranchConflict(null);
              } catch (e) {
                console.error("commit_and_checkout:", e);
                setBranchConflict((current) => current ? {
                  ...current,
                  busy: false,
                  error: t("branches.commitSwitchError"),
                } : current);
                return;
              }
            })();
          }}
        />
      )}
      {githubAuth.open && (
        <BranchGithubAuthDialog
          state={githubAuth.state}
          onCancel={githubAuth.cancel}
          onConnect={() => void githubAuth.connect()}
        />
      )}
    </>
  );
}
