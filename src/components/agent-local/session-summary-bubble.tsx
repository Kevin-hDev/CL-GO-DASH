import { useEffect, useMemo, useRef, useState } from "react";
import { useTranslation } from "react-i18next";
import { ClipboardText, FilePlus, GitBranch } from "@/components/ui/icons";
import { Tooltip } from "@/components/ui/tooltip";
import { useClickOutside } from "@/hooks/use-click-outside";
import type { useSessionSummary } from "@/hooks/use-session-summary";
import type { AgentPlanRun } from "@/types/agent";
import {
  SessionSummaryPlanList,
  SessionSummarySubagentList,
  SessionSummaryTodoList,
} from "./session-summary-lists";
import { SessionSummarySection } from "./session-summary-section";
import "./session-summary-bubble.css";

type SessionSummaryState = ReturnType<typeof useSessionSummary>;

export interface SessionSummaryGitState {
  isGitRepo: boolean;
  isLoading: boolean;
  currentBranch: string;
  worktrees: { branch: string; path: string; is_current: boolean }[];
}

interface SessionSummaryBubbleProps {
  summary: SessionSummaryState;
  git?: SessionSummaryGitState;
  onOpenPlan?: (plan: AgentPlanRun) => void;
  onOpenSubagent?: (sessionId: string) => void;
}

type SectionKey = "todos" | "plans" | "subagents";

export function SessionSummaryBubble({ summary, git, onOpenPlan, onOpenSubagent }: SessionSummaryBubbleProps) {
  const { t } = useTranslation();
  const [open, setOpen] = useState(false);
  const [sections, setSections] = useState<Record<SectionKey, boolean>>({
    todos: false,
    plans: false,
    subagents: false,
  });
  const rootRef = useRef<HTMLSpanElement>(null);
  useClickOutside(rootRef, () => setOpen(false));

  const branch = useMemo(() => branchLabel(git, t), [git, t]);
  const toggleSection = (key: SectionKey) => {
    setSections((current) => ({ ...current, [key]: !current[key] }));
  };

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key !== "Escape") return;
      event.preventDefault();
      event.stopPropagation();
      event.stopImmediatePropagation();
      setOpen(false);
    };
    window.addEventListener("keydown", onKeyDown, { capture: true });
    return () => window.removeEventListener("keydown", onKeyDown, { capture: true });
  }, [open]);

  return (
    <span className="ssb-root" ref={rootRef}>
      <Tooltip label={t("agentLocal.sessionSummary.tooltip")} align="right">
        <button
          className={`tab-action-btn ${open ? "active" : ""}`}
          type="button"
          aria-label={t("agentLocal.sessionSummary.tooltip")}
          aria-expanded={open}
          onClick={(event) => {
            event.stopPropagation();
            setOpen((value) => !value);
          }}
        >
          <ClipboardText size="var(--chrome-icon-md)" />
        </button>
      </Tooltip>
      {open && (
        <div className="ssb-popover" role="dialog" aria-label={t("agentLocal.sessionSummary.title")}>
          <div className="ssb-kicker">{t("agentLocal.sessionSummary.environment")}</div>
          <div className="ssb-row">
            <FilePlus size="var(--icon-md)" className="ssb-row-icon" />
            <span className="ssb-row-label">{t("agentLocal.sessionSummary.modifications")}</span>
            <ChangeStats additions={summary.changes.additions} deletions={summary.changes.deletions} />
          </div>
          <div className="ssb-row">
            <GitBranch size="var(--icon-md)" className="ssb-row-icon" />
            <span className="ssb-row-label">{t("agentLocal.sessionSummary.branch")}</span>
            <span className="ssb-row-value" title={branch}>{branch}</span>
          </div>
          <div className="ssb-separator" />
          <SessionSummarySection title={t("agentLocal.sessionSummary.sections.todos")} count={summary.todoRuns.length} open={sections.todos} onToggle={() => toggleSection("todos")}>
            <SessionSummaryTodoList runs={summary.todoRuns} />
          </SessionSummarySection>
          <SessionSummarySection title={t("agentLocal.sessionSummary.sections.plans")} count={summary.plans.length} open={sections.plans} onToggle={() => toggleSection("plans")}>
            <SessionSummaryPlanList
              plans={summary.plans}
              onOpenPlan={(plan) => {
                setOpen(false);
                onOpenPlan?.(plan);
              }}
            />
          </SessionSummarySection>
          <SessionSummarySection title={t("agentLocal.sessionSummary.sections.subagents")} count={summary.subagents.length} open={sections.subagents} onToggle={() => toggleSection("subagents")}>
            <SessionSummarySubagentList
              subagents={summary.subagents}
              onOpen={(id) => {
                setOpen(false);
                onOpenSubagent?.(id);
              }}
            />
          </SessionSummarySection>
        </div>
      )}
    </span>
  );
}

function ChangeStats({ additions, deletions }: { additions: number; deletions: number }) {
  const { t } = useTranslation();
  if (additions === 0 && deletions === 0) {
    return <span className="ssb-row-value">{t("agentLocal.sessionSummary.noChanges")}</span>;
  }
  return (
    <span className="ssb-change-stats">
      <span className="ssb-change-add">+{additions}</span>
      <span className="ssb-change-del">-{deletions}</span>
    </span>
  );
}

function branchLabel(git: SessionSummaryGitState | undefined, t: (key: string) => string): string {
  if (!git) return t("agentLocal.sessionSummary.noGit");
  if (git.isLoading && !git.currentBranch) return t("common.loading");
  if (!git.isGitRepo) return t("agentLocal.sessionSummary.noGit");
  return git.currentBranch || t("branches.detachedHead");
}
