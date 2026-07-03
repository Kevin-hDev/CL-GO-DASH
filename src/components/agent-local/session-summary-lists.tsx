import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CheckCircle2, ChevronDown, Circle, Clock3, PauseCircle } from "@/components/ui/lucide-icons";
import { FileText, Users } from "@/components/ui/icons";
import type { AgentPlanRun, AgentTodoItem, AgentTodoRun, SubagentInfo } from "@/types/agent";
import "./session-summary-tasks.css";

export function SessionSummaryTodoList({ runs }: { runs: AgentTodoRun[] }) {
  const { t } = useTranslation();
  const [openRunIds, setOpenRunIds] = useState<Set<string>>(() => new Set());
  if (runs.length === 0) {
    return <EmptyLine label={t("agentLocal.sessionSummary.emptyTodos")} />;
  }
  return runs.map((run) => {
    const done = run.todos.filter((todo) => todo.status === "completed").length;
    const open = openRunIds.has(run.id);
    const statusLabel = t(`agentLocal.sessionSummary.todoRunStatus.${run.status}`);
    const StatusIcon = run.status === "paused" ? PauseCircle : Clock3;
    return (
      <div className="ssb-run" key={run.id}>
        <button
          className="ssb-item ssb-item-button"
          type="button"
          aria-expanded={open}
          onClick={() => setOpenRunIds((current) => {
            const next = new Set(current);
            if (next.has(run.id)) next.delete(run.id);
            else next.add(run.id);
            return next;
          })}
        >
          <span
            aria-label={statusLabel}
            className={`ssb-item-icon ssb-item-status ssb-item-status-${run.status}`}
            role="img"
            title={statusLabel}
          >
            <StatusIcon aria-hidden="true" size="var(--icon-sm)" />
          </span>
          <span className="ssb-item-main" title={run.title}>{run.title}</span>
          <span className="ssb-item-meta">
            {t("agentLocal.sessionSummary.todoProgress", { done, total: run.todos.length })}
          </span>
          <ChevronDown className={`ssb-item-caret ${open ? "ssb-item-caret-open" : ""}`} aria-hidden="true" />
        </button>
        <div className={`ssb-accordion ${open ? "ssb-accordion-open" : ""}`}>
          <div className="ssb-accordion-inner">
            <div className="ssb-task-list">
              {run.todos.map((todo, index) => (
                <TodoTaskRow key={`${todo.status}-${index}-${todo.content}`} todo={todo} />
              ))}
            </div>
          </div>
        </div>
      </div>
    );
  });
}

export function SessionSummaryPlanList({
  plans,
  onOpenPlan,
}: {
  plans: AgentPlanRun[];
  onOpenPlan?: (plan: AgentPlanRun) => void;
}) {
  const { t } = useTranslation();
  if (plans.length === 0) {
    return <EmptyLine label={t("agentLocal.sessionSummary.emptyPlans")} />;
  }
  return plans.map((plan) => (
    <button
      className="ssb-item ssb-item-button"
      key={plan.id}
      type="button"
      onClick={() => onOpenPlan?.(plan)}
    >
      <FileText size="var(--icon-sm)" className="ssb-item-icon" />
      <span className="ssb-item-main" title={plan.title}>{plan.title}</span>
      <span className="ssb-item-meta">
        {t(`agentLocal.sessionSummary.planStatus.${plan.status}`)}
      </span>
    </button>
  ));
}

export function SessionSummarySubagentList({
  subagents,
  onOpen,
}: {
  subagents: SubagentInfo[];
  onOpen: (sessionId: string) => void;
}) {
  const { t } = useTranslation();
  if (subagents.length === 0) {
    return <EmptyLine label={t("agentLocal.sessionSummary.emptySubagents")} />;
  }
  return subagents.map((agent) => (
    <button
      className="ssb-item ssb-item-button"
      key={agent.sessionId}
      type="button"
      onClick={() => onOpen(agent.sessionId)}
    >
      <Users size="var(--icon-sm)" className="ssb-item-icon" />
      <span className="ssb-item-main" title={agent.name}>
        {t(`agentLocal.sessionSummary.subagentType.${agent.type}`)} · {agent.name}
      </span>
      <span className="ssb-item-meta">{t(`subagents.${agent.status}`)}</span>
    </button>
  ));
}

function EmptyLine({ label }: { label: string }) {
  return <div className="ssb-empty">{label}</div>;
}

function TodoTaskRow({ todo }: { todo: AgentTodoItem }) {
  const { t } = useTranslation();
  const Icon = todo.status === "completed"
    ? CheckCircle2
    : todo.status === "in_progress" ? Clock3 : Circle;
  const text = todo.status === "in_progress" && todo.active_form
    ? todo.active_form
    : todo.content;

  return (
    <div className={`ssb-task ssb-task-${todo.status}`}>
      <Icon className="ssb-task-icon" aria-hidden="true" />
      <span className="ssb-task-text" title={text}>{text}</span>
      <span className="ssb-task-status">{t(`todos.status.${todo.status}`)}</span>
    </div>
  );
}
