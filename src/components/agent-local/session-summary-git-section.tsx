import { useCallback, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, GitBranch } from "@/components/ui/icons";
import { BranchGithubAuthDialog } from "./branch-github-auth-dialog";
import { GitCommitDialog } from "./git-commit-dialog";
import { useGithubBranchAuth } from "@/hooks/use-github-branch-auth";
import type { GitActionResult, GitDirtyFile, GitPushTarget } from "@/hooks/git-types";

export interface SessionSummaryGitState {
  repositoryPath: string;
  isGitRepo: boolean;
  isLoading: boolean;
  currentBranch: string;
  dirtyCount: number;
  hasRemote: boolean;
  isGithubRemote: boolean;
  hasRemoteBranch: boolean;
  aheadCount: number;
  behindCount: number;
  worktrees: { branch: string; path: string; is_current: boolean }[];
  listDirtyFiles: () => Promise<GitDirtyFile[]>;
  commit: (description?: string) => Promise<GitActionResult>;
  push: (target: GitPushTarget) => Promise<GitActionResult>;
  refresh: () => Promise<void>;
}

interface SessionSummaryGitSectionProps {
  git?: SessionSummaryGitState;
}

export function SessionSummaryGitSection({ git }: SessionSummaryGitSectionProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [commitOpen, setCommitOpen] = useState(false);
  const [files, setFiles] = useState<GitDirtyFile[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string>();
  const pendingPushTarget = useRef<GitPushTarget | undefined>(undefined);
  const branch = branchLabel(git, t);
  const retryPush = useCallback(async () => {
    const target = pendingPushTarget.current;
    pendingPushTarget.current = undefined;
    if (!git || !target) return;
    setBusy(true);
    setError(undefined);
    const result = await git.push(target);
    if (!result.ok) setError(pushErrorLabel(result.kind, t));
    setBusy(false);
  }, [git, t]);
  const githubAuth = useGithubBranchAuth(() => void retryPush());

  if (!git?.isGitRepo) {
    return <StaticBranchRow branch={branch} />;
  }

  const status = statusLabel(git, t);
  const openCommit = async () => {
    setFiles(await git.listDirtyFiles());
    setError(undefined);
    setCommitOpen(true);
  };
  const commit = async (description?: string) => {
    if (busy) return;
    setBusy(true);
    const result = await git.commit(description);
    if (result.ok) setCommitOpen(false);
    else setError(t("agentLocal.sessionSummary.git.commitError"));
    setBusy(false);
  };
  const push = async () => {
    if (busy) return;
    setBusy(true);
    setError(undefined);
    const target = { repositoryPath: git.repositoryPath, branch: git.currentBranch };
    const result = await git.push(target);
    if (!result.ok) {
      if (result.kind === "authentication_required" && git.isGithubRemote) {
        pendingPushTarget.current = target;
        githubAuth.request();
      }
      else setError(pushErrorLabel(result.kind, t));
    }
    setBusy(false);
  };

  return (
    <>
      <button
        className="ssb-row ssb-git-toggle"
        type="button"
        aria-label={t("agentLocal.sessionSummary.git.toggle", { branch })}
        aria-expanded={open}
        onClick={() => setOpen((value) => !value)}
      >
        <GitBranch size="var(--icon-md)" className="ssb-row-icon" />
        <span className="ssb-row-label">{t("agentLocal.sessionSummary.branch")}</span>
        <span className="ssb-row-value" title={branch}>{branch}</span>
        <CaretDown className={`ssb-section-caret ${open ? "ssb-section-caret-open" : ""}`} size="var(--icon-sm)" />
      </button>
      <div className={`ssb-accordion ${open ? "ssb-accordion-open" : ""}`}>
        <div className="ssb-accordion-inner">
          <div className="ssb-git-panel">
            <div className="ssb-git-status">{status}</div>
            {git.dirtyCount > 0 && (
              <button className="ssb-git-action" type="button" onClick={() => void openCommit()} disabled={busy}>
                {t("agentLocal.sessionSummary.git.commit")}
              </button>
            )}
            {git.dirtyCount === 0
              && git.hasRemote
              && git.behindCount === 0
              && (!git.hasRemoteBranch || git.aheadCount > 0) && (
              <button className="ssb-git-action" type="button" onClick={() => void push()} disabled={busy}>
                {t("agentLocal.sessionSummary.git.push")}
              </button>
            )}
            {error && !commitOpen && <div className="ssb-git-error">{error}</div>}
          </div>
        </div>
      </div>
      {commitOpen && (
        <GitCommitDialog files={files} busy={busy} error={error} onCancel={() => setCommitOpen(false)} onCommit={(description) => void commit(description)} />
      )}
      {githubAuth.open && (
        <BranchGithubAuthDialog state={githubAuth.state} onCancel={githubAuth.cancel} onConnect={() => void githubAuth.connect()} />
      )}
    </>
  );
}

function StaticBranchRow({ branch }: { branch: string }) {
  const { t } = useTranslation();
  return (
    <div className="ssb-row">
      <GitBranch size="var(--icon-md)" className="ssb-row-icon" />
      <span className="ssb-row-label">{t("agentLocal.sessionSummary.branch")}</span>
      <span className="ssb-row-value" title={branch}>{branch}</span>
    </div>
  );
}

function branchLabel(git: SessionSummaryGitState | undefined, t: (key: string) => string) {
  if (!git) return t("agentLocal.sessionSummary.noGit");
  if (git.isLoading && !git.currentBranch) return t("common.loading");
  if (!git.isGitRepo) return t("agentLocal.sessionSummary.noGit");
  return git.currentBranch || t("branches.detachedHead");
}

function statusLabel(git: SessionSummaryGitState, t: (key: string, values?: Record<string, unknown>) => string) {
  if (git.dirtyCount > 0) return t("agentLocal.sessionSummary.git.changesToCommit", { count: git.dirtyCount });
  if (!git.hasRemote) return t("agentLocal.sessionSummary.git.noRemote");
  if (git.behindCount > 0) return t("agentLocal.sessionSummary.git.remoteChanges");
  if (git.aheadCount > 0) {
    return t("agentLocal.sessionSummary.git.commitsToPush", {
      count: git.aheadCount,
      branch: git.currentBranch,
      destination: t(git.isGithubRemote
        ? "agentLocal.sessionSummary.git.destinationGithub"
        : "agentLocal.sessionSummary.git.destinationRemote"),
    });
  }
  if (!git.hasRemoteBranch) {
    return t("agentLocal.sessionSummary.git.localBranch", { branch: git.currentBranch });
  }
  return t("agentLocal.sessionSummary.git.upToDate");
}

function pushErrorLabel(kind: string, t: (key: string) => string) {
  const keys: Record<string, string> = {
    no_remote: "agentLocal.sessionSummary.git.noRemote",
    authentication_required: "agentLocal.sessionSummary.git.authenticationError",
    permission_denied: "agentLocal.sessionSummary.git.permissionDenied",
    remote_changed: "agentLocal.sessionSummary.git.remoteChanged",
    network_unavailable: "agentLocal.sessionSummary.git.networkError",
    context_changed: "agentLocal.sessionSummary.git.contextChanged",
  };
  return t(keys[kind] ?? "agentLocal.sessionSummary.git.pushError");
}
