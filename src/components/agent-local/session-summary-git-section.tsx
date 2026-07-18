import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CaretDown, GitBranch } from "@/components/ui/icons";
import { BranchGithubAuthDialog } from "./branch-github-auth-dialog";
import { GitCommitDialog } from "./git-commit-dialog";
import { useGithubBranchAuth } from "@/hooks/use-github-branch-auth";
import type { GitActionResult, GitDirtyFile } from "@/hooks/git-types";

export interface SessionSummaryGitState {
  isGitRepo: boolean;
  isLoading: boolean;
  currentBranch: string;
  dirtyCount: number;
  hasRemote: boolean;
  isGithubRemote: boolean;
  hasUpstream: boolean;
  aheadCount: number;
  behindCount: number;
  worktrees: { branch: string; path: string; is_current: boolean }[];
  listDirtyFiles: () => Promise<GitDirtyFile[]>;
  commit: (description?: string) => Promise<GitActionResult>;
  push: () => Promise<GitActionResult>;
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
  const githubAuth = useGithubBranchAuth(() => void git?.refresh());
  const branch = branchLabel(git, t);

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
    const result = await git.push();
    if (!result.ok) {
      if (result.kind === "authentication_required" && git.isGithubRemote) githubAuth.request();
      else setError(t(result.kind === "remote_changed"
        ? "agentLocal.sessionSummary.git.remoteChanged"
        : "agentLocal.sessionSummary.git.pushError"));
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
                {t("agentLocal.sessionSummary.git.save")}
              </button>
            )}
            {git.dirtyCount === 0 && git.hasRemote && (!git.hasUpstream || git.aheadCount > 0) && (
              <button className="ssb-git-action" type="button" onClick={() => void push()} disabled={busy}>
                {t(git.hasUpstream
                  ? "agentLocal.sessionSummary.git.push"
                  : "agentLocal.sessionSummary.git.publish")}
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

function statusLabel(git: SessionSummaryGitState, t: (key: string, values?: Record<string, number>) => string) {
  if (git.dirtyCount > 0) return t("agentLocal.sessionSummary.git.changesToSave", { count: git.dirtyCount });
  if (!git.hasRemote) return t("agentLocal.sessionSummary.git.noRemote");
  if (!git.hasUpstream) return t("agentLocal.sessionSummary.git.notPublished");
  if (git.aheadCount > 0) return t("agentLocal.sessionSummary.git.commitsToPush", { count: git.aheadCount });
  if (git.behindCount > 0) return t("agentLocal.sessionSummary.git.remoteChanges");
  return t("agentLocal.sessionSummary.git.upToDate");
}
