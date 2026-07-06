import { useCallback, useEffect, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { branchCreateErrorKey } from "@/components/agent-local/branch-selector-utils";
import { parseCreateBranchError, type useGitBranch } from "@/hooks/use-git-branch";
import { showToast } from "@/lib/toast-emitter";
import type { SessionTab } from "@/types/agent";

export type CloneGitBranchActionState = "idle" | "loading" | "success" | "linked";

interface Options {
  projectPath?: string;
  git: ReturnType<typeof useGitBranch>;
  isStreaming: boolean;
  activeSessionTab?: SessionTab | null;
  onCreateCloneGitBranch?: (path: string, cloneSessionId: string) => Promise<string>;
}

export function useCloneGitBranchAction({
  projectPath,
  git,
  isStreaming,
  activeSessionTab,
  onCreateCloneGitBranch,
}: Options) {
  const { t } = useTranslation();
  const [rawState, setRawState] = useState<{
    sessionId: string | null;
    status: "idle" | "loading" | "success";
  }>({ sessionId: null, status: "idle" });
  const state = rawState.sessionId === activeSessionTab?.session_id ? rawState.status : "idle";

  useEffect(() => {
    if (!rawState.sessionId || rawState.sessionId === activeSessionTab?.session_id) return;
    const timer = window.setTimeout(() => setRawState({ sessionId: null, status: "idle" }), 0);
    return () => window.clearTimeout(timer);
  }, [activeSessionTab?.session_id, rawState.sessionId]);

  useEffect(() => {
    if (rawState.status !== "success") return;
    const timer = window.setTimeout(() => {
      setRawState({ sessionId: null, status: "idle" });
    }, 3000);
    return () => window.clearTimeout(timer);
  }, [rawState.status]);

  const isCloneTab = !!activeSessionTab
    && !activeSessionTab.is_main
    && !!activeSessionTab.clone_parent_session_id;
  const eligible = !!projectPath
    && !!onCreateCloneGitBranch
    && isCloneTab
    && !activeSessionTab.git_branch
    && git.isGitRepo;
  const linked = isCloneTab && !!activeSessionTab?.git_branch && git.isGitRepo;
  const visible = eligible || linked || state === "loading" || state === "success";
  const effectiveState: CloneGitBranchActionState = linked && state !== "success" ? "linked" : state;
  const disabled = isStreaming || effectiveState !== "idle";
  const branchSelectorLockedLabel = eligible
    ? t("agentLocal.clone.gitNoIsolation")
    : undefined;

  const label = useMemo(() => {
    if (effectiveState === "linked") return activeSessionTab?.git_branch ?? "";
    if (effectiveState === "loading") return t("agentLocal.clone.gitCreating");
    if (effectiveState === "success") return t("agentLocal.clone.gitCreated");
    return t("agentLocal.clone.gitCreate");
  }, [activeSessionTab?.git_branch, effectiveState, t]);

  const onCreate = useCallback(() => {
    if (disabled || !eligible || !activeSessionTab || !projectPath || !onCreateCloneGitBranch) {
      return;
    }
    void (async () => {
      setRawState({ sessionId: activeSessionTab.session_id, status: "loading" });
      try {
        await onCreateCloneGitBranch(projectPath, activeSessionTab.session_id);
        await git.refresh();
        setRawState({ sessionId: activeSessionTab.session_id, status: "success" });
      } catch (error) {
        const kind = parseCreateBranchError(error);
        showToast(t(branchCreateErrorKey(kind ?? undefined)), "error", 3000);
        setRawState({ sessionId: activeSessionTab.session_id, status: "idle" });
      }
    })();
  }, [activeSessionTab, disabled, eligible, git, onCreateCloneGitBranch, projectPath, t]);

  return { visible, state: effectiveState, label, disabled, branchSelectorLockedLabel, onCreate };
}
