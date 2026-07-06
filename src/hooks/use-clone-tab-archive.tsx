import { useCallback, useState } from "react";
import { useTranslation } from "react-i18next";
import { CloneGitArchiveDialog } from "@/components/agent-local/clone-git-archive-dialog";
import { showToast } from "@/lib/toast-emitter";
import type { SessionTab, SessionTabs } from "@/types/agent";

const BRANCH_LINKED_TO_OTHER_SESSION_ERROR = "clone_branch_linked_to_other_session";

interface Options {
  tabs: SessionTabs | null;
  projectPath?: string;
  onCloseTab: (tabId: string) => Promise<void>;
  onCloseTabWithGitCleanup: (tabId: string, path: string, fallbackBranch?: string) => Promise<void>;
  getMainBranch: () => string | undefined;
}

export function useCloneTabArchive({
  tabs,
  projectPath,
  onCloseTab,
  onCloseTabWithGitCleanup,
  getMainBranch,
}: Options) {
  const { t } = useTranslation();
  const [pending, setPending] = useState<SessionTab | null>(null);
  const [busy, setBusy] = useState(false);

  const closeTab = useCallback((tabId: string) => {
    const tab = tabs?.tabs.find((item) => item.tab_id === tabId);
    if (!tab) return;
    if (tab.git_branch) {
      setPending(tab);
      return;
    }
    void onCloseTab(tabId);
  }, [onCloseTab, tabs]);

  const archive = useCallback(() => {
    if (!pending) return;
    setBusy(true);
    void onCloseTab(pending.tab_id)
      .then(() => setPending(null))
      .catch(() => showToast(t("errors.operationFailed"), "error"))
      .finally(() => setBusy(false));
  }, [onCloseTab, pending, t]);

  const cleanupArchive = useCallback(() => {
    if (!pending || !projectPath) return;
    const fallbackBranch = getMainBranch();
    if (!fallbackBranch) {
      showToast(t("agentLocal.clone.gitMissingCheckpoint"), "error", 3000);
      return;
    }
    setBusy(true);
    void onCloseTabWithGitCleanup(pending.tab_id, projectPath, fallbackBranch)
      .then(() => setPending(null))
      .catch((err) => showToast(t(cleanupErrorKey(err)), "error", 3000))
      .finally(() => setBusy(false));
  }, [getMainBranch, onCloseTabWithGitCleanup, pending, projectPath, t]);

  const dialog = pending ? (
    <CloneGitArchiveDialog
      branchName={pending.git_branch ?? ""}
      busy={busy}
      onArchive={archive}
      onCancel={() => { if (!busy) setPending(null); }}
      onCleanupArchive={cleanupArchive}
    />
  ) : null;

  return { closeTab, dialog };
}

function cleanupErrorKey(err: unknown): string {
  if (
    err === BRANCH_LINKED_TO_OTHER_SESSION_ERROR
    || (err instanceof Error && err.message === BRANCH_LINKED_TO_OTHER_SESSION_ERROR)
  ) {
    return "agentLocal.clone.gitBranchLinkedElsewhere";
  }
  return "errors.operationFailed";
}
