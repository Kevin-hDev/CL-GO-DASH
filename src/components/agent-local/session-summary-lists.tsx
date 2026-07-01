import { useTranslation } from "react-i18next";
import { ClipboardText, FileText, Users } from "@/components/ui/icons";
import type { AgentPlanRun, AgentTodoRun, SubagentInfo } from "@/types/agent";

export function SessionSummaryTodoList({ runs }: { runs: AgentTodoRun[] }) {
  const { t } = useTranslation();
  if (runs.length === 0) {
    return <EmptyLine label={t("agentLocal.sessionSummary.emptyTodos")} />;
  }
  return runs.map((run) => {
    const done = run.todos.filter((todo) => todo.status === "completed").length;
    return (
      <div className="ssb-item" key={run.id}>
        <ClipboardText size="var(--icon-sm)" className="ssb-item-icon" />
        <span className="ssb-item-main" title={run.title}>{run.title}</span>
        <span className="ssb-item-meta">
          {t("agentLocal.sessionSummary.todoProgress", { done, total: run.todos.length })}
        </span>
      </div>
    );
  });
}

export function SessionSummaryPlanList({ plans }: { plans: AgentPlanRun[] }) {
  const { t } = useTranslation();
  if (plans.length === 0) {
    return <EmptyLine label={t("agentLocal.sessionSummary.emptyPlans")} />;
  }
  return plans.map((plan) => (
    <div className="ssb-item" key={plan.id}>
      <FileText size="var(--icon-sm)" className="ssb-item-icon" />
      <span className="ssb-item-main" title={plan.title}>{plan.title}</span>
      <span className="ssb-item-meta">
        {t(`agentLocal.sessionSummary.planStatus.${plan.status}`)}
      </span>
    </div>
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
